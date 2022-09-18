use std::marker::PhantomData;

use druid::{Widget, Event, EventCtx, LifeCycleCtx, LifeCycle, UpdateCtx, Env, LayoutCtx, BoxConstraints, PaintCtx, Size, Data};

/// Similar to a `LensWrap`, but does not require a mutable reference.
pub struct NonMutWrap<T, U, A, W> {
    child: W,
    accessor: A,
    /// The inner data type.
    phantom_u: PhantomData<U>,
    /// The outer data type.
    phantom_t: PhantomData<T>,
}

impl<T, U, A, W> NonMutWrap<T, U, A, W> {
    pub fn new(child: W, accessor: A) -> Self {
        Self {
            child,
            accessor,
            phantom_u: PhantomData,
            phantom_t: PhantomData,
        }
    }
}

impl<T, U, A, W> Widget<T> for NonMutWrap<T, U, A, W>
    where T: Data,
          U: Data,
          A: Fn(&T) -> U,
          W: Widget<U> {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut T, _env: &Env) {
        // Swallow events
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.child.lifecycle(ctx, event, &(self.accessor)(data), env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.child.update(ctx, &(self.accessor)(old_data), &(self.accessor)(data), env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        self.child.layout(ctx, bc, &(self.accessor)(data), env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.child.paint(ctx, &(self.accessor)(data), env)
    }
}

/// A trait for convenience. Implemented by widgets that can be wrapped in a `NonMutWrap`.
pub trait NonMutWrappable<T, U, A> {
    type Wrapped;

    fn nonmut_wrap(self, accessor: A) -> Self::Wrapped;
}

impl<T, U, A, W> NonMutWrappable<T, U, A> for W {
    type Wrapped = NonMutWrap<T, U, A, Self>;

    fn nonmut_wrap(self, accessor: A) -> Self::Wrapped {
        NonMutWrap::new(self, accessor)
    }
}
