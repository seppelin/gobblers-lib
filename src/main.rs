use gobblers::search::Search;

fn main() {
    let mut s = Search::new();
    s.pre_evaluate(4, 10);
    s.flush();
}
