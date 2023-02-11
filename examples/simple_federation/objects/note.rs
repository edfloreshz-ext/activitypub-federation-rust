use crate::{error::Error, generate_object_id, instance::DatabaseHandle, objects::person::MyUser};
use activitypub_federation::{
    core::object_id::ObjectId,
    deser::helpers::deserialize_one_or_many,
    request_data::RequestData,
    traits::ApubObject,
};
use activitystreams_kinds::{object::NoteType, public};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug)]
pub struct MyPost {
    pub text: String,
    pub ap_id: ObjectId<MyPost>,
    pub creator: ObjectId<MyUser>,
    pub local: bool,
}

impl MyPost {
    pub fn new(text: String, creator: ObjectId<MyUser>) -> MyPost {
        MyPost {
            text,
            ap_id: ObjectId::new(generate_object_id(creator.inner().domain().unwrap()).unwrap()),
            creator,
            local: true,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "type")]
    kind: NoteType,
    id: ObjectId<MyPost>,
    pub(crate) attributed_to: ObjectId<MyUser>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    content: String,
}

#[async_trait::async_trait]
impl ApubObject for MyPost {
    type DataType = DatabaseHandle;
    type ApubType = Note;
    type DbType = ();
    type Error = Error;

    async fn read_from_apub_id(
        _object_id: Url,
        _data: &RequestData<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        todo!()
    }

    async fn into_apub(
        self,
        data: &RequestData<Self::DataType>,
    ) -> Result<Self::ApubType, Self::Error> {
        let creator = self.creator.dereference_local(data).await?;
        Ok(Note {
            kind: Default::default(),
            id: self.ap_id,
            attributed_to: self.creator,
            to: vec![public(), creator.followers_url()?],
            content: self.text,
        })
    }

    async fn from_apub(
        apub: Self::ApubType,
        data: &RequestData<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        let post = MyPost {
            text: apub.content,
            ap_id: apub.id,
            creator: apub.attributed_to,
            local: false,
        };

        let mut lock = data.posts.lock().unwrap();
        lock.push(post.clone());
        Ok(post)
    }
}