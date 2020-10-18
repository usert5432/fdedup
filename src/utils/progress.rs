use indicatif::{ProgressBar, ProgressStyle};

pub fn get_progress_bar(size : u64, title : &str, show_progress : bool)
    -> Option<ProgressBar>
{

    if ! show_progress {
        return None;
    }

    let result = ProgressBar::new(size);
    result.set_style(
        ProgressStyle::default_bar()
            .template(
                "{msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] \
                {pos}/{len} ({percent:>2}%) (ETA: {eta})"
            )
            .progress_chars("#>-")
    );

    result.set_message(title);

    Some(result)
}

