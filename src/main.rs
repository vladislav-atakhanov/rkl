fn main() -> Result<(), String> {
    let content = std::fs::read_to_string("./keyboards/imperial44.txt")
        .map_err(|_| "Parse Error".to_string())?;
    let keyboard = parser::parse(content.as_str())?;
    Ok(())
}
