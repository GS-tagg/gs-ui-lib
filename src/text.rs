use std::fs;
fn read_font(){
    let font_data = fs::read_to_string(file_path).expect("failed to read font");
    println!(font_data)
}