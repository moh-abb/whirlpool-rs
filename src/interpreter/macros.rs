#[macro_export]
macro_rules! impl_frame {
    (
        $(#[$inst_attr:meta])*
        $iv:vis struct $inst_name:ident {
            $(#[$inst_state_attr:meta])*
            $inst_state_name:ident: $inst_state_type:ty,
            $(
                $(#[$field_attr:meta])*
                $field_name:ident: $field_type:ty
            ),*
            $(,)?
        },
        [$($inst_types:tt)*]
        = [$($inst_clauses_fst:ident = $inst_clauses_snd:ident),*]
        + [$($inst_eval_clauses_fst:ty = $inst_eval_clauses_snd:ident),*],

        $(#[$state_attr:meta])*
        $sv:vis enum $state_name:ident {},
        [$($state_types:tt)*]
        = [$($state_clauses:tt)*],

        [$self_name:ident, $stack_name:ident, $arenas_name:ident],
        $(
            $(#[$cur_state_attr:meta])*
            fn $cur_state:ident
                (
                    $(
                        $cur_state_field:ident : $cur_state_type:ty
                    ),*
                    $(,)?
                )
                $next_state:block
        ),*
        $(,)?
    ) => {
        $(#[$inst_attr])*
        $iv struct $inst_name<$($inst_types)*>
        where $($inst_clauses_fst: $inst_clauses_snd,)*
        {
            $(#[$inst_state_attr])*
            $inst_state_name: $inst_state_type,
            $(
                $(#[$field_attr])*
                $field_name: $field_type,
            )*
        }

        #[allow(non_camel_case_types)]
        $(#[$state_attr])*
        $sv enum $state_name<$($state_types)*>
            where $($state_clauses)*
        {
            finish,
            $(
                $(#[$cur_state_attr])*
                $cur_state
                    {
                        $(
                            $cur_state_field: $cur_state_type,
                        )*
                    },
            )*
        }

        impl<$($inst_types)*> crate::interpreter::frame::EvaluateFrame
            for $inst_name<$($inst_types)*>
            where
                $($inst_eval_clauses_fst: $inst_eval_clauses_snd,)*
                $($inst_clauses_fst: $inst_clauses_snd,)*
        {
            fn step(
                $self_name: &mut Self,
                $stack_name: &mut impl crate::interpreter::value::ValueStack,
                $arenas_name: &impl crate::ast::expr::linked::ExprArenas,
            ) -> crate::interpreter::frame::InterpreterResult {
                use $state_name::*;
                loop {
                    let cur_state = $self_name.$inst_state_name.clone();
                    let (next_state, opt_next_frame) = match cur_state {
                        finish => (finish, None),
                        $(
                            $cur_state
                                {
                                    $(
                                        $cur_state_field,
                                    )*
                                }
                                => $next_state,
                        )*
                    };
                    $self_name.$inst_state_name = next_state;
                    if let finish = &$self_name.state {
                        break Ok(opt_next_frame);
                    }
                    if let Some(frame) = opt_next_frame {
                        break Ok(Some(frame));
                    }
                    // Continue to find the next frame.
                }
            }
        }
    }
}
