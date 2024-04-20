use clipboard::{ClipboardContext, ClipboardProvider};
use std::{io, time::Duration};


// Display visited locations on a map
fn display(visited_locations: Vec<LocationRecord>, width: usize) {
    println!("");
    // Find largest absolute x or z coordinate
    let max_x = visited_locations.iter().map(|location_record| location_record.location.x.abs()).fold(0.0, f64::max);
    let max_z = visited_locations.iter().map(|location_record| location_record.location.z.abs()).fold(0.0, f64::max);
    let max = f64::max(max_x, max_z) + 512.0;

    let num_chars_x = width;
    let num_chars_y = width / 2;
    let mut board = vec![vec![' '; num_chars_x]; num_chars_y];
    // Add axes
    let center_x = num_chars_x / 2;
    let center_y = num_chars_y / 2;

    // Iterate over chars in board and mark if you're within 512 blocks of a location on either axis
    for x in 0..num_chars_x {
        for z in 0..num_chars_y {
            let x_coord = (x as f64 - center_x as f64) * max / center_x as f64;
            let z_coord = (z as f64 - center_y as f64) * max / center_y as f64;
            for location_record in visited_locations.iter() {
                // Filter for overworld locations
                if let Dimension::Overworld = location_record.location.dimension {
                    let dx = location_record.location.x - x_coord;
                    let dz = location_record.location.z - z_coord;
                    if dx.abs() < 512.0 && dz.abs() < 512.0 {
                        board[z as usize][x as usize] = 'X';
                    }
                }
            }
        }
    }

    // Print board, using white for the axes, green for the player's location, and red for visited locations
    // Also print axis labels at center, 50% of max, and max

    // Print top axis labels
    // 5 digits (2222 => 2.2k, 22222 =>  22k)
    fn format_num(num: i32) -> String {
        if num < 1000 {
            return format!(" {}", num);
        } else if num < 10_000 {
            let mut num_str = (num as f32 / 1000.0).to_string();
            num_str.truncate(3);
            return format!("{}k", num_str);
        } else {
            let mut num_str = (num as f32 / 1000.0).to_string();
            num_str.truncate(2);
            return format!(" {}k", num_str);
        }
    }
    let max_label_str = format_num(max as i32);
    let half_label_str = format_num((max / 2.0) as i32);
    let spacing = num_chars_x / 4 - 2;
    let top_label = format!(
        "     -{}{}-{}{}{}{}{}{}{}", max_label_str, " ".repeat(spacing-1),
        half_label_str, " ".repeat(spacing-1), "0", " ".repeat(spacing), 
        half_label_str, " ".repeat(spacing), max_label_str
    );
    println!("{}", top_label);
    println!("{}-X-", " ".repeat(num_chars_x / 2 + 8));

    // Print board, and axes
    for i in 0..num_chars_y {
        if i == center_y {
            println!("0    -Z- {}+{} +Z+", "-".repeat(num_chars_x / 2), "-".repeat(num_chars_x / 2));
            continue;
        }
        let mut row = String::new();
        
        // Left axis label
        if i == 0 {
            row.push_str("-");
            row.push_str(max_label_str.as_str());
        } else if i == num_chars_y / 4 {
            row.push_str("-");
            row.push_str(half_label_str.as_str());
        } else if i == 3 * num_chars_y / 4 {
            row.push_str(" ");
            row.push_str(half_label_str.as_str());
        } else if i == num_chars_y - 1 {
            row.push_str(" ");
            row.push_str(max_label_str.as_str());
        } else {
            row.push_str("     ");
        }

        row.push_str("    ");
        for j in 0..num_chars_x / 2 {
            row.push(board[i][j]);
        }
        row.push_str(&format!("|"));
        for j in num_chars_x / 2..num_chars_x {
            row.push(board[i][j]);
        }

        println!("{}", row);
    }
    println!("{}+X+", " ".repeat(num_chars_x / 2 + 8));


}

#[derive(Clone, Debug)]
pub enum Dimension {
    Overworld,
    Nether,
    End,
}

#[derive(Clone, Debug)]
pub struct Location {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub dimension: Dimension,
}

#[derive(Clone, Debug)]
pub enum Method {
    F3C,
    F3I,
}

#[derive(Clone, Debug)]
pub struct LocationRecord {
    pub location: Location,
    pub method: Method,
}


// F3 + C
// /execute in minecraft:overworld run tp @s -246.50 69.00 -18.50 0.00 0.00

// F3 + I
// /setblock -250 68 -17 minecraft:grass_block[snowy=false]
// Returns the location and whether the location is in the overworld
fn extract_location(clipboard_content: String) -> Option<LocationRecord> {
    // Check for F3 + C
    if clipboard_content.starts_with("/execute in") {
        let parts: Vec<&str> = clipboard_content.split_whitespace().collect();
        if parts.len() >= 9 {
            let x = parts[6].parse::<f64>().ok()?;
            let y = parts[7].parse::<f64>().ok()?;
            let z = parts[8].parse::<f64>().ok()?;
            let location = Location {
                x,
                y,
                z,
                dimension:
                    match parts[2] {
                        "minecraft:overworld" => Dimension::Overworld,
                        "minecraft:the_nether" => Dimension::Nether,
                        "minecraft:the_end" => Dimension::End,
                        _ => return None,
                    }
            };
            return Some(LocationRecord {
                location,
                method: Method::F3C,
            });
        } else {
            return None;
        }
    } else if clipboard_content.starts_with("/setblock") {
        let parts: Vec<&str> = clipboard_content.split_whitespace().collect();
        if parts.len() >= 4 {
            let x = parts[1].parse::<f64>().ok()?;
            let y = parts[2].parse::<f64>().ok()?;
            let z = parts[3].parse::<f64>().ok()?;
            let location = Location {
                x,
                y,
                z,
                dimension: Dimension::Overworld,
            };
            return Some(LocationRecord {
                location,
                method: Method::F3I,
            });
        } else {
            return None;
        }
    } else {
        return None;
    }
}

// Repeatedly check the clipboard content every second
// Extract location from clipboard content, keep track of
// visited locations and display them on a map
fn main() {
    // Read width from terminal
    println!("Enter width of map (lowest = 20): ");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let width: usize = input.trim().parse().expect("Please enter a valid number");
    let width = if width < 20 { 20 } else { width };

    let mut visited_locations = Vec::new();
    let mut previous_clipboard_content = String::new();

    loop {
        // Read clipboard content
        let clipboard_content = match ClipboardContext::new() {
            Ok(mut ctx) => ctx.get_contents().unwrap_or_else(|_| String::new()),
            Err(_) => String::new(),
        };

        // Check if the content has changed
        if clipboard_content != previous_clipboard_content {
            previous_clipboard_content = clipboard_content.clone();

            // Extract location from clipboard content
            if let Some(location_record) = extract_location(clipboard_content.clone()) {
                // Display location on map
                visited_locations.push(location_record.clone());
                display(visited_locations.clone(), width);
            }
        }


        // Wait for one second before checking again
        std::thread::sleep(Duration::from_secs(1));
    }
}
