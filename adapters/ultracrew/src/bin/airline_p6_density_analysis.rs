use std::fs::File;
use std::io::Write;
use ultracrew::models::{Shift, Worker, Skill};
use ultracrew::partitioning::{Partitioner, TemporalPartitioner};

fn generate_family_c(weekend_ratio: f64) -> Vec<Shift> {
    let skill = "Pilot".to_string();
    let total_hours = 1140; // 95% of 1200
    let weekend_hours = (total_hours as f64 * weekend_ratio) as u64;
    let weekday_hours = total_hours - weekend_hours;
    
    let weekend_shifts = weekend_hours / 8;
    let weekday_shifts = weekday_hours / 8;
    
    let mut shifts = vec![];
    for i in 0..weekend_shifts {
        shifts.push(Shift { id: (i + 1) as u64, start_hour: 120 + ((i * 8) % 40), duration_hours: 8, required_skill: Skill(skill.clone()) });
    }
    for i in 0..weekday_shifts {
        shifts.push(Shift { id: (weekend_shifts + i + 1) as u64, start_hour: (i * 8) % 120, duration_hours: 8, required_skill: Skill(skill.clone()) });
    }
    shifts.sort_by_key(|s| s.start_hour);
    shifts
}

fn main() -> std::io::Result<()> {
    let global_shifts = generate_family_c(0.60);
    let global_workers: Vec<Worker> = (0..40).map(|id| Worker { id, skills: vec![] }).collect();
    
    // Use P2 Baseline architecture to evaluate the 8 static partitions
    let partitioner = TemporalPartitioner {
        num_partitions: 8,
        halo_hours: 24,
    };
    
    let partitions = partitioner.partition(&global_shifts, &global_workers);
    
    let mut file = File::create("p6_density_analysis.csv")?;
    writeln!(file, "partition_id,is_weekend,local_feasible_history,core_shifts_count,total_shift_hours,max_concurrent_shifts,mean_concurrent_shifts,core_conflict_edges,crossover_conflict_edges")?;
    
    for p in &partitions {
        let core = &p.core_shifts;
        let halo = &p.halo_shifts;
        
        // 1. Total shift hours
        let total_shift_hours: u64 = core.iter().map(|s| s.duration_hours).sum();
        
        // Find core temporal bounds
        let min_start = core.iter().map(|s| s.start_hour).min().unwrap_or(0);
        let max_end = core.iter().map(|s| s.start_hour + s.duration_hours).max().unwrap_or(0);
        let duration = if max_end > min_start { max_end - min_start } else { 1 };
        
        // 2 & 3. Concurrency
        let mut max_concurrent = 0;
        let mut sum_concurrent = 0;
        for h in min_start..max_end {
            let mut active = 0;
            for s in core {
                if h >= s.start_hour && h < s.start_hour + s.duration_hours {
                    active += 1;
                }
            }
            if active > max_concurrent {
                max_concurrent = active;
            }
            sum_concurrent += active;
        }
        let mean_concurrent = sum_concurrent as f64 / duration as f64;
        
        // 5. Core conflict edges (overlapping + rest violations)
        // Two shifts conflict if they overlap OR gap < 8 hours
        let mut core_conflict_edges = 0;
        for i in 0..core.len() {
            for j in (i+1)..core.len() {
                let s1 = &core[i];
                let s2 = &core[j];
                
                let s1_end = s1.start_hour + s1.duration_hours;
                let s2_end = s2.start_hour + s2.duration_hours;
                
                let overlap = !(s1_end <= s2.start_hour || s2_end <= s1.start_hour);
                let rest_violation = !overlap && (
                    (s1_end <= s2.start_hour && s2.start_hour - s1_end < 8) ||
                    (s2_end <= s1.start_hour && s1.start_hour - s2_end < 8)
                );
                
                if overlap || rest_violation {
                    core_conflict_edges += 1;
                }
            }
        }
        
        // 6. Crossover conflict edges
        let mut crossover_conflict_edges = 0;
        for s1 in core {
            for s2 in halo {
                let s1_end = s1.start_hour + s1.duration_hours;
                let s2_end = s2.start_hour + s2.duration_hours;
                
                let overlap = !(s1_end <= s2.start_hour || s2_end <= s1.start_hour);
                let rest_violation = !overlap && (
                    (s1_end <= s2.start_hour && s2.start_hour - s1_end < 8) ||
                    (s2_end <= s1.start_hour && s1.start_hour - s2_end < 8)
                );
                
                if overlap || rest_violation {
                    crossover_conflict_edges += 1;
                }
            }
        }
        
        // Identify if weekend (partitions 5,6,7)
        let is_weekend = p.id >= 5;
        let local_feasible_history = if is_weekend { "0%" } else { "100%" };
        
        writeln!(
            file, 
            "{},{},{},{},{},{},{:.2},{},{}",
            p.id,
            is_weekend,
            local_feasible_history,
            core.len(),
            total_shift_hours,
            max_concurrent,
            mean_concurrent,
            core_conflict_edges,
            crossover_conflict_edges
        )?;
        
        println!("Partition {} | Feasible: {} | Core Shifts: {:>3} | Shift Hours: {:>3} | Max Concurr: {:>2} | Mean Concurr: {:>5.2} | Core Edges: {:>4} | Crossover Edges: {:>4}",
                 p.id, local_feasible_history, core.len(), total_shift_hours, max_concurrent, mean_concurrent, core_conflict_edges, crossover_conflict_edges);
    }
    
    Ok(())
}
