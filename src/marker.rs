pub trait Sealed {}

//------------------------------------------------------------------------------

pub trait Static {}

impl Static for StaticInput {}

pub struct StaticInput;

pub struct DynamicInput;
