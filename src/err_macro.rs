
#[macro_export]
macro_rules! sloppy_unwrap_or {
    ( $x:expr, $state:expr, $msg:expr, $err_handle:expr ) => {
        match $x {
            Ok(x)  => x,
            Err(e) => {
                if $state.abort_on_error {
                    if $msg.len() > 0 {
                        error!("{} : {}", $msg, e);
                    }

                    return Err(e);
                }
                else {
                    if $msg.len() > 0 {
                        warn!("{} : {}", $msg, e);
                    }

                    $err_handle
                }
            }
        }
    };
}

macro_rules! verbose_question_mark {
    ( $x:expr, $state:expr, $msg:expr ) => {
        match $x {
            Ok(x)  => x,
            Err(e) => {
                if $state.abort_on_error {
                    error!("{} : {}", $msg, e);
                }
                else {
                    warn!("{} : {}", $msg, e);
                }

                return Err(e);
            }
        }
    };
}

macro_rules! sloppy_unwrap_or_continue {
    ( $x:expr, $abort:expr, $msg:expr ) => {
        sloppy_unwrap_or!($x, $abort, $msg, continue)
    };
}

