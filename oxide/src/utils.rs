
/// Extract `target` from the given pattern if it matches, otherwise return `err`.
/// 
/// This macro should be used when a failing match means there's an error in the source code.
#[macro_export]
macro_rules! match_or {
    ($pattern:pat = $expr:expr, $target:expr, $err:expr) => {
        if let $pattern = $expr {
            $target
        } else {
            $err
        }
    };
}

/// Extract `target` from the given pattern if it matches, otherwise panic with a message.
/// 
/// This macro should be used only when the pattern is guaranteed to match. If it doesn't, it's an internal bug.
#[macro_export]
macro_rules! match_unreachable {
    ($pattern:pat = $expr:expr, $target:expr) => {
        {
            let __tmp = $expr;
            if let $pattern = __tmp {
                $target
            } else {
                unreachable!("This pattern should always match, but expression evaluates to {:?}. This is a bug.", __tmp)
            }
        }
    };
}

