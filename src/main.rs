use std::{process::{Command, Stdio}, fs::File, path::Path, error::Error, io::{Write, BufReader, BufRead}, cmp::Ordering};
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use rand::Rng;

struct Run{
    perfect_clears: usize,
    path: String,
}
impl Ord for Run{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl PartialOrd for Run{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.perfect_clears.partial_cmp(&other.perfect_clears)
    }
}
impl Eq for Run{}
impl PartialEq for Run{
    fn eq(&self, other: &Self) -> bool {
        self.perfect_clears == other.perfect_clears
    }
}

fn gen_seed() -> usize{
    rand::thread_rng().gen_range(0..(2_usize.pow(32)))
}

fn main() {
    let runs = match std::env::var("RUNS"){
        Ok(var) => var.parse::<usize>().unwrap_or(10),
        Err(_) => 100,
    };
    let max_saved_runs = match std::env::var("MAX_SAVED_RUNS"){
        Ok(var) => var.parse::<usize>().unwrap_or(10),
        Err(_) => 10,
    };

    eprintln!("attempting {runs} runs while saving the top {max_saved_runs}");

    let mut run_heap: BinaryHeap<Reverse<Run>> = BinaryHeap::new();

    for _ in 0..runs{
        let mut seed;
        let mut path;
        loop{
            seed = gen_seed();
            path = format!("runs/{seed}.txt");
            if !Path::new(&path).exists(){
                break;
            }
        }
        let file = File::create(&path).expect(&format!("able to create file at {}", path));
        let stdio = Stdio::from(file);
        if let Ok(pcs) = run(seed, &path, stdio){
            let new_run = Reverse(Run{
                perfect_clears: pcs,
                path
            });
            if run_heap.len()==max_saved_runs {
                let worst_run= run_heap.pop().unwrap();
                if worst_run.0.cmp(&new_run.0)==Ordering::Less{
                    std::fs::remove_file(worst_run.0.path).unwrap();
                    run_heap.push(new_run);
                }else{
                    run_heap.push(worst_run);
                }
            }else{
                run_heap.push(new_run);
            }
        }else{
            eprintln!("FAILED RUN");
        }
    }
    let mut runs = run_heap.into_sorted_vec();
    runs.reverse();
    if runs.len() == 0 {
        eprintln!("NO RUNS");
    }
    for run in runs{
        println!("{}pcs @{}", run.0.perfect_clears, run.0.path);
    }
}


fn run(seed: usize, path: &str, stdio: Stdio)-> Result<usize, Box<dyn Error>>{
    let mut command = Command::new("./hydra_bot.out")
    .current_dir("../hydra")
    .stdin(Stdio::piped())
    .stdout(stdio)
    .spawn()?;

    let mut stdin = command.stdin.take().ok_or("no stdin")?;

    std::thread::spawn(move || -> std::io::Result<()>{
        stdin.write_all(seed.to_string().as_bytes())?;
        Ok(())
    });
    
    command.wait()?;

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let lines_iter = reader.lines().scan((), |_,line|line.ok());
    let pcs = lines_iter.last().ok_or("no lines from reader")?;
    eprintln!("pcs {pcs}");
    let pcs : usize = pcs.parse()?;
    Ok(pcs)
}