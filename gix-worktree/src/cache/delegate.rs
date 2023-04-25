use crate::cache::State;
use crate::PathIdMapping;

pub struct StackDelegate<'a, Find> {
    pub state: &'a mut State,
    pub buf: &'a mut Vec<u8>,
    pub is_dir: bool,
    pub id_mappings: &'a Vec<PathIdMapping>,
    pub find: Find,
    pub case: gix_glob::pattern::Case,
}

impl<'a, Find, E> gix_fs::stack::Delegate for StackDelegate<'a, Find>
where
    Find: for<'b> FnMut(&gix_hash::oid, &'b mut Vec<u8>) -> Result<gix_object::BlobRef<'b>, E>,
    E: std::error::Error + Send + Sync + 'static,
{
    fn push_directory(&mut self, stack: &gix_fs::Stack) -> std::io::Result<()> {
        match &mut self.state {
            State::CreateDirectoryAndAttributesStack { attributes, .. } => {
                attributes.push_directory(
                    stack.root(),
                    stack.current(),
                    self.buf,
                    self.id_mappings,
                    &mut self.find,
                    self.case,
                )?;
            }
            State::AttributesAndIgnoreStack { ignore, attributes } => {
                attributes.push_directory(
                    stack.root(),
                    stack.current(),
                    self.buf,
                    self.id_mappings,
                    &mut self.find,
                    self.case,
                )?;
                ignore.push_directory(
                    stack.root(),
                    stack.current(),
                    self.buf,
                    self.id_mappings,
                    &mut self.find,
                    self.case,
                )?
            }
            State::IgnoreStack(ignore) => ignore.push_directory(
                stack.root(),
                stack.current(),
                self.buf,
                self.id_mappings,
                &mut self.find,
                self.case,
            )?,
        }
        Ok(())
    }

    fn push(&mut self, is_last_component: bool, stack: &gix_fs::Stack) -> std::io::Result<()> {
        match &mut self.state {
            State::CreateDirectoryAndAttributesStack {
                #[cfg(debug_assertions)]
                test_mkdir_calls,
                unlink_on_collision,
                attributes: _,
            } => {
                #[cfg(debug_assertions)]
                {
                    create_leading_directory(
                        is_last_component,
                        stack,
                        self.is_dir,
                        test_mkdir_calls,
                        *unlink_on_collision,
                    )?
                }
                #[cfg(not(debug_assertions))]
                {
                    create_leading_directory(is_last_component, stack, self.is_dir, *unlink_on_collision)?
                }
            }
            State::AttributesAndIgnoreStack { .. } | State::IgnoreStack(_) => {}
        }
        Ok(())
    }

    fn pop_directory(&mut self) {
        match &mut self.state {
            State::CreateDirectoryAndAttributesStack { attributes, .. } => {
                attributes.pop_directory();
            }
            State::AttributesAndIgnoreStack { attributes, ignore } => {
                attributes.pop_directory();
                ignore.pop_directory();
            }
            State::IgnoreStack(ignore) => {
                ignore.pop_directory();
            }
        }
    }
}

fn create_leading_directory(
    is_last_component: bool,
    stack: &gix_fs::Stack,
    is_dir: bool,
    #[cfg(debug_assertions)] mkdir_calls: &mut usize,
    unlink_on_collision: bool,
) -> std::io::Result<()> {
    if is_last_component && !is_dir {
        return Ok(());
    }
    #[cfg(debug_assertions)]
    {
        *mkdir_calls += 1;
    }
    match std::fs::create_dir(stack.current()) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            let meta = stack.current().symlink_metadata()?;
            if meta.is_dir() {
                Ok(())
            } else if unlink_on_collision {
                if meta.file_type().is_symlink() {
                    gix_fs::symlink::remove(stack.current())?;
                } else {
                    std::fs::remove_file(stack.current())?;
                }
                #[cfg(debug_assertions)]
                {
                    *mkdir_calls += 1;
                }
                std::fs::create_dir(stack.current())
            } else {
                Err(err)
            }
        }
        Err(err) => Err(err),
    }
}
