use std::borrow::Cow;


pub struct StaticID(usize);


pub enum StaticValue<'a> {
    String(Cow<'a, str>)
}


pub struct StaticsTable<'a> {

    values: Vec<StaticValue<'a>>

}

impl<'a> StaticsTable<'a> {

    pub fn new() -> Self {
        Self {
            values: Vec::new()
        }
    }

    pub fn declare_static(&mut self, value: StaticValue<'a>) -> StaticID {
        let id = self.values.len();
        self.values.push(value);
        StaticID(id)
    }


    pub fn get_static(&self, id: StaticID) -> &StaticValue<'a> {
        // Guaranteed to be within bounds because StaticsTable is the only entity that can issue StaticIDs
        &self.values[id.0]
    }

}
