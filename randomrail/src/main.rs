mod infrastructure;
pub mod oldmain;
pub mod timetable;
use infrastructure::generate_infrastructure;
use timetable::{generate_timetable, Timetable};

fn main() {
    let (infrastructure, services, bottleneck) = generate_infrastructure();
    std::fs::write("i1.json", serde_json::to_string(&infrastructure).unwrap()).unwrap();
    let timetable: Timetable = generate_timetable(&infrastructure, &services, bottleneck);
    std::fs::write("tt1.json", serde_json::to_string(&timetable).unwrap()).unwrap();
}
