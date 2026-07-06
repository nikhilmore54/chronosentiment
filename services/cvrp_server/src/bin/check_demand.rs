use cvrp::CvrpInstance;

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let total_demand: i32 = instance.customers.iter().map(|c| c.demand).sum();
    println!("Total Customers: {}", instance.customers.len());
    println!("Total Demand: {}", total_demand);
    println!("Capacity: {}", instance.capacity);
    println!("Minimum possible vehicles: {}", (total_demand as f64 / instance.capacity as f64).ceil());
}
