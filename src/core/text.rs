use crate::core::font::Font;

/// Calculate how many lines are needed for text with wrapping
/// Implements character-level breaking for long words
pub fn calculate_text_lines(text: &str, width: f64, size: f64, font: &Font) -> usize {
    if text.is_empty() {
        return 1;
    }
    
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut buffer = Vec::new();
    let mut line_count = 0;
    
    for word in words {
        // Check if word alone is wider than available width
        let word_width = font.measure_text(word, size);
        
        if word_width > width {
            // Word needs character-level breaking
            // First, count the current buffer as a line if not empty
            if !buffer.is_empty() {
                line_count += 1;
                buffer.clear();
            }
            
            // Count lines needed for this word broken at character level
            let chars: Vec<char> = word.chars().collect();
            let mut char_buffer = String::new();
            
            for ch in chars {
                let test_str = format!("{}{}", char_buffer, ch);
                let test_width = font.measure_text(&test_str, size);
                
                if test_width <= width {
                    char_buffer.push(ch);
                } else {
                    if !char_buffer.is_empty() {
                        line_count += 1;
                    }
                    char_buffer.clear();
                    char_buffer.push(ch);
                }
            }
            
            // Count the last character buffer line
            if !char_buffer.is_empty() {
                line_count += 1;
            }
        } else {
            // Try adding this word to the buffer
            let mut test_line = buffer.clone();
            test_line.push(word);
            let test_text = test_line.join(" ");
            let test_width = font.measure_text(&test_text, size);
            
            if test_width <= width {
                // Word fits, add it to buffer
                buffer.push(word);
            } else {
                // Word doesn't fit
                if !buffer.is_empty() {
                    // Complete the current line
                    line_count += 1;
                    buffer.clear();
                }
                // Start new line with this word
                buffer.push(word);
            }
        }
    }
    
    // Count the last line
    if !buffer.is_empty() {
        line_count += 1;
    }
    
    line_count.max(1) // At least 1 line
}

/// Split text into two parts: one that fits in max_lines, and the remainder.
/// Returns (Head, Tail). Tail is None if all fits.
pub fn split_text_at_lines(text: &str, width: f64, size: f64, font: &Font, max_lines: usize) -> (String, Option<String>) {
    if max_lines == 0 {
        return (String::new(), Some(text.to_string()));
    }

    // Reuse logic from calculate_text_lines but track byte index
    let words: Vec<&str> = text.split_whitespace().collect();
    // let mut buffer = Vec::new(); // Unused in split version
    let mut current_lines = 1;
    let mut consumed_words = 0;
    
    // Naive re-implementation for MVP (ideally we refactor to shared iterator)
    // We will build the "Head" string.
    let mut head_str = String::new();
    let mut word_iter = words.iter().peekable();
    
    // We need to reconstruct the string carefully or just return String.
    // Let's use the buffer approach to build lines.
    
    // Logic: Fill buffer. When line is full, flush buffer to head_str. 
    // If lines > max_lines, stop and return rest.
    
    let mut line_buffer = Vec::new();

    while let Some(&word) = word_iter.peek() {
        // word is &&str here because peek returns &Item
        
        let word_width = font.measure_text(word, size);
        
        // Check if word fits in current line
        let mut test_line = line_buffer.clone();
        test_line.push(*word);
        let test_text = test_line.join(" ");
        let test_width = font.measure_text(&test_text, size);
        
        if test_width <= width {
            // Fits
            line_buffer.push(*word);
            word_iter.next(); 
        } else {
            // Doesn't fit.
            if line_buffer.is_empty() {
                // Word is wider than line. Forced break.
                line_buffer.push(*word);
                word_iter.next();
            }
            
            // Flush current line
            if !head_str.is_empty() {
                head_str.push(' ');
            }
            head_str.push_str(&line_buffer.join(" "));
            line_buffer.clear();
            
            if current_lines >= max_lines {
                break; 
            }
            current_lines += 1;
        }
    }
    
    // If loop finished (all words consumed)
    if !line_buffer.is_empty() {
         if current_lines <= max_lines {
             if !head_str.is_empty() { head_str.push(' '); }
             head_str.push_str(&line_buffer.join(" "));
             return (head_str, None);
         }
    } else {
        // Buffer empty, meaning we flushed exactly at boundary?
        return (head_str, Some(collect_rest(word_iter)));
    }
    
    // If we broke early
    if word_iter.peek().is_some() || !line_buffer.is_empty() {
        // Remainder
        let mut tail = if !line_buffer.is_empty() { line_buffer.join(" ") } else { String::new() };
        let rest = collect_rest(word_iter);
        if !rest.is_empty() {
            if !tail.is_empty() { tail.push(' '); }
             tail.push_str(&rest);
        }
        return (head_str, Some(tail));
    }

    (head_str, None)
}

fn collect_rest(mut iter: std::iter::Peekable<std::slice::Iter<&str>>) -> String {
    let mut s = String::new();
    while let Some(&w) = iter.next() {
        if !s.is_empty() { s.push(' '); }
        s.push_str(w);
    }
    s
}
