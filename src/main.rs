// Implement basic function to split some generic computational work between threads
// Split should occur only on some threshold
// If computational work is shorter that this threshold
//     No splitting should occur and no threads should be created

// You get as input: 

// 1. Vec<T>
// 2. Function f(t: T) -> R

// You should return:
//    1. Up to you, but probably some Vec of the same length as input(1)


use num_cpus;

use std::time::{Duration, Instant};
use std::thread::{spawn};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{sleep};
use std::mem;


const MAX_NO_THREADS_DURATION: Duration = Duration::from_secs(1);


fn split_work_run_threads<R, T>(input: &[T], func: fn(T) -> R, result: Vec<R>) -> Vec<R>
where
    R: Clone + Send + Sync + 'static,
    T: Clone + Send + 'static
{
    let threads_available = num_cpus::get();
    let threads_count = if input.len() < threads_available {input.len()} else {threads_available};
    let calls_count_approx = input.len() / threads_count;
    let mut calls_count_remainder = input.len() % threads_count;

    println!("Starting {} threads for {} calls", threads_count, input.len());

    let sync_push = Arc::new(AtomicU8::new(0));
    let sync_result = Arc::new(Mutex::new(result));

    let mut calculated = 0;
    let mut threads = vec![];

    while calculated < input.len()
    {
        let thread_id = threads.len() as u8;

        let calls_count = calls_count_approx + if calls_count_remainder > 0 {1} else {0};
        println!("Thread {}: prepares to start for {} calls", thread_id, calls_count);

        let thread_input = input.get(calculated..calculated + calls_count).unwrap().to_owned();
        
        let sync_push_cpy = sync_push.clone();
        let sync_result_cpy = sync_result.clone();

        threads.push(spawn(move ||
        {
            println!("Thread {}: started", thread_id);
            let mut thread_result = vec![];
            for arg in thread_input
            {
                thread_result.push(func(arg.clone()));
            }
            println!("Thread {}: waits", thread_id);
            while sync_push_cpy.as_ref().load(Ordering::Acquire) != thread_id
            {
                sleep(MAX_NO_THREADS_DURATION / 100);
            }
            println!("Thread {}: inserts", thread_id);
            let result_ref = &mut *sync_result_cpy.as_ref().lock().unwrap(); // safe unwrap cause of atomic guard
            for r in thread_result
            {
                result_ref.push(r.clone());
            }
            sync_push_cpy.as_ref().store(thread_id + 1, Ordering::Release);
            println!("Thread {}: finished", thread_id);
        }));

        calculated += calls_count;
        calls_count_remainder -= if calls_count_remainder > 0 {1} else {0};
    }
    for t in threads
    {
        t.join().unwrap();
    }
    let result = mem::take(&mut *sync_result.as_ref().lock().unwrap());
    return result;
}

fn split_work<R, T>(input: &Vec<T>, func: fn(T) -> R) -> Vec<R>
where
    R: Clone + Send + Sync + 'static,
    T: Clone + Send + 'static
{
    println!("Splitting work for {} calls", input.len());

    let mut result = Vec::<R>::with_capacity(input.capacity());
    let mut calculated = 0;
    let start_time = Instant::now();

    while calculated < input.len() && start_time.elapsed() < MAX_NO_THREADS_DURATION
    {
        let arg = input.get(calculated).unwrap().clone();
        result.push(func(arg));
        calculated += 1;
    }
    println!("{} was calculated without spawning threads", calculated);

    if calculated == input.len()
    {
        return result;
    }
    return split_work_run_threads(input.get(calculated..input.len()).unwrap(), func, result);
}

fn sum(end: i64) -> i64
{
    let mut result = 0_i64;
    for i in 0..=end
    {
        result += i;
    }
    sleep(MAX_NO_THREADS_DURATION / 100);
    return result;
}

fn main()
{
    let mut input = Vec::<i64>::new();
    for i in 1..=100
    {
        input.push(i);
    }
    let result = split_work(&mut input, sum);
    for i in 0..100
    {
        println!("{}", result[i]);
    }
}
