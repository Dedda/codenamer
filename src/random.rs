use rand::seq::SliceRandom;
use rand::Rng;

pub trait GetRandom<T> {

    fn get_random(self) -> Option<T>;

    fn get_n_random(self, n: usize) -> Vec<T>;
}

impl<T> GetRandom<T> for &Vec<T> where T: Clone {
    fn get_random(self) -> Option<T> {
        self.choose(&mut rand::thread_rng()).cloned()
    }

    fn get_n_random(self, n: usize) -> Vec<T> where T: Clone {
        if n > self.len() {
            panic!("Trying to get more elements out of vec than it contains!")
        }
        let mut indices: Vec<usize> = (0..self.len()).collect();
        let mut rng = rand::thread_rng();
        let mut selected: Vec<T> = Vec::new();
        for _ in 0..n {
            let index = rng.gen_range(0, indices.len());
            let index = indices.remove(index);
            selected.push(self.get(index).unwrap().clone())
        }
        selected
    }
}