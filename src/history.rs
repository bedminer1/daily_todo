use std::{env, fs, io::Write, path::PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn skip(now: &str, next: &[&str]) -> Result<()> {
    let (&new_now, rest) = next.split_first().ok_or("No NEXT task to swap with")?;
    let mut new_next = Vec::with_capacity(next.len());
    new_next.push(now);
    new_next.extend_from_slice(rest);

    let (source_path, updated) = updated_source(new_now, &new_next)?;
    let temp_path = project_dir().join("src/main.rs.tmp");
    fs::write(&temp_path, updated)?;
    fs::rename(temp_path, source_path)?;

    println!("NOW: {new_now}");
    println!("NEXT: {now}");
    Ok(())
}

pub fn done(now: &str, next: &[&str]) -> Result<()> {
    if now.is_empty() {
        return Err("There is no NOW task to finish".into());
    }

    let (new_now, new_next) = match next.split_first() {
        Some((&first, rest)) => (first, rest.to_vec()),
        None => ("", vec![]),
    };
    let (source_path, updated) = updated_source(new_now, &new_next)?;
    let temp_path = project_dir().join("src/main.rs.tmp");
    fs::write(&temp_path, updated)?;

    let singapore = chrono::FixedOffset::east_opt(8 * 60 * 60).unwrap();
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(project_dir().join("done.txt"))?;
    writeln!(
        log,
        "{} | {now}",
        chrono::Utc::now()
            .with_timezone(&singapore)
            .format("%Y-%m-%d %H:%M")
    )?;
    fs::rename(temp_path, source_path)?;

    println!("Done: {now}");
    println!(
        "Now: {}",
        if new_now.is_empty() {
            "Nothing queued"
        } else {
            new_now
        }
    );
    Ok(())
}

pub fn undo(now: &str, next: &[&str]) -> Result<()> {
    let log_path = project_dir().join("done.txt");
    let log = fs::read_to_string(&log_path)?;
    let trimmed = log.trim_end_matches('\n');
    let (remaining, last) = trimmed
        .rsplit_once('\n')
        .map(|(remaining, last)| (format!("{remaining}\n"), last))
        .unwrap_or_else(|| (String::new(), trimmed));
    let (_, restored) = last.split_once(" | ").ok_or("No completed task to undo")?;

    let mut new_next = Vec::with_capacity(next.len() + 1);
    if !now.is_empty() {
        new_next.push(now);
    }
    new_next.extend_from_slice(next);
    let (source_path, updated) = updated_source(restored, &new_next)?;
    let temp_path = project_dir().join("src/main.rs.tmp");
    fs::write(&temp_path, updated)?;
    fs::write(&log_path, remaining)?;
    fs::rename(temp_path, source_path)?;

    println!("Undone: {restored}");
    println!("Now: {restored}");
    Ok(())
}

fn project_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn updated_source(now: &str, next: &[&str]) -> Result<(PathBuf, String)> {
    let source_path = project_dir().join("src/main.rs");
    let source = fs::read_to_string(&source_path)?;
    let start = source
        .find("const NOW: &str = ")
        .ok_or("NOW declaration missing")?;
    let next_start = source[start..]
        .find("const NEXT: [&str; ")
        .ok_or("NEXT declaration missing")?
        + start;
    let end = source[next_start..]
        .find("];")
        .map(|offset| next_start + offset + 2)
        .ok_or("NEXT list ending missing")?;
    let replacement = format!(
        "const NOW: &str = {now:?};\nconst NEXT: [&str; {}] = [\n{}];",
        next.len(),
        next.iter()
            .map(|task| format!("    {task:?},\n"))
            .collect::<String>()
    );
    Ok((
        source_path,
        format!("{}{}{}", &source[..start], replacement, &source[end..]),
    ))
}
