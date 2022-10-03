// --------------------
//
// Cwago.
//
// cwago/cwago_ecs/src/ecs.rs
// (C) 2022 Taichi Ito.
// ====================

//! ECSシステム提供します。

use std::fmt::Display;

use crate::{
    ent::Id,
    comp::{
        data::{
            self,
            Chunk,
            Data
        },
        ty::{
            self,
            Archetype,
            Request,
            Datas
        }
    }
};

/// ECSシステムです。
pub struct ECS {

    ids: Vec<Option<IdInfo>>, // Id情報リストです。
    chunks: Vec<Chunk>,       // チャンクリストです。
    free_ids: Vec<Id>,        // 再利用待ちIdのスタックです。
}
impl ECS {

    /// 生成します。
    pub fn new() -> Self {

        ECS { 
            ids: Vec::new(), 
            chunks: Vec::new(), 
            free_ids: Vec::new() 
        }
    }

    /// 実体を生成します。
    /// 
    /// # Arguments 
    /// 
    /// * `args` - コンポーネントデータの初期値です。
    /// 
    pub fn spawn(&mut self, args: &Datas) -> Result<Id, Error> {

        match self.spawn_id() {
            Ok(id) => {
                
                match self.spawn_data(&id, &args) {

                    Ok((chunk_idx, data_idx)) => {
                        
                        self.insert_id(&id, chunk_idx, data_idx);
                        Ok(id)
                    },
                    Err(err) => Err(err),
                }
            },
            Err(err) => Err(err),
        }
    }

    /// 実体を削除します。
    /// 
    /// # Arguments
    /// 
    /// * `id` - 削除する実体のIdです。
    /// 
    pub fn despawn(&mut self, id: &Id) -> Result<(), Error> {
        
        // idをリストから削除します。
        match self.remove_id(id) {
            
            Ok((chunk_idx, datas_idx)) => {
                
                // フリーリストに追加します。
                self.despawn_id(id);
                
                self.despawn_data(chunk_idx, datas_idx)
            },
            Err(err) => Err(err)
        }
    }
    
    /// 新たにデータを追加します。
    /// 
    /// # Arguments 
    /// 
    /// * `id` - 追加する対象実体のIdです。
    /// * `args` - コンポーネントデータの初期値です。
    /// 
    pub fn attach(&mut self, id: &Id, args: &Datas) -> Result<(), Error> {
        
        // 位置を取得します。
        let (old_chunk_idx, old_datas_idx) = match self.get_position(id) {
            
            Ok(idx) => idx,
            Err(err) => return Err(err),
        };
        
        // 元データを取得し、削除します。
        let mut datas = match self.pop_data(old_chunk_idx, old_datas_idx) {
            
            Ok(datas) => datas,
            Err(err) => return Err(err),
        };
        
        // 初期値データを統合します。
        datas.append(args);
        
        // 再生成します。
        let (new_chunk_idx, new_datas_idx) = match self.spawn_data(id, &datas) {
            
            Ok(pos) => pos,
            Err(err) => return Err(err),
        };
        
        // 位置を再設定します。
        self.set_position(id, new_chunk_idx, new_datas_idx)
    }

    /// 一部のデータを削除します。
    /// 
    /// # Arguments 
    /// 
    /// * `id` - 削除する対象実体のIdです。
    /// * `req` - 削除するデータのリクエストです。
    /// 
    pub fn deattach(&mut self, id: &Id, req: &Request) -> Result<Datas, Error> {

        // 位置を取得します。
        let (old_chunk_idx, old_datas_idx) = match self.get_position(id) {
            
            Ok(idx) => idx,
            Err(err) => return Err(err),
        };
        
        // 元データを取得し、削除します。
        let mut datas = match self.pop_data(old_chunk_idx, old_datas_idx) {
            
            Ok(datas) => datas,
            Err(err) => return Err(err),
        };
        
        // 初期値データを統合します。
        let datas = match datas.split(req) {

            Ok(datas) => datas,
            Err(err) => return Err(Error::ProcessError(data::Error::TypeError(err))),
        };
        
        // 再生成します。
        let (new_chunk_idx, new_datas_idx) = match self.spawn_data(id, &datas) {
            
            Ok(pos) => pos,
            Err(err) => return Err(err),
        };
        
        // 位置を再設定します。
        match self.set_position(id, new_chunk_idx, new_datas_idx) {
            
            Ok(_) => Ok(datas),
            Err(err) => Err(err),
        }
    }

    /// フリーIdを取得します。
    fn spawn_id(&mut self) -> Result<Id, Error> {

        if let Some(id) = self.free_ids.pop() {
            
            let id = id.increment();
            Ok(id)

        } else {

            if self.ids.len() <= u32::MAX as usize {
                
                Ok(
                    Id::new(self.ids.len() as u32)
                )
            
            } else {

                Err(Error::IdOverflow)
            }
        }
    }
    
    /// フリーIdに設定します。
    ///
    /// # Arguments
    ///
    /// * `id` - 設定するEntityのIdです。
    ///
    fn despawn_id(&mut self, id: &Id) {
        
        self.free_ids.push(id.clone());
    }

    /// Idリストに追加します。 
    ///
    /// # Arguments
    ///
    /// * `id` - 追加するEntityのIdです。
    /// * `chunk_idx` - チャンクの位置です。
    /// * `data_idx` - データの位置です。
    /// 
    fn insert_id(&mut self, id: &Id, chunk_idx: usize, data_idx: usize) {

        let info = IdInfo::new(id, chunk_idx, data_idx);
        if id.index() < self.ids.len() {
    
            self.ids[id.index()] = Some(info);

        } else {

            self.ids.push(Some(info));
        }
    }
    
    /// Idリストから削除します。
    ///
    /// # Arguments
    ///
    /// * `id` - 削除するEntityのIdです。
    ///
    fn remove_id(&mut self, id: &Id) -> Result<(usize, usize), Error> {
        
        match self.get_position(id) {
            
            Ok(res) => {

                self.ids[id.index()] = None;
                Ok(res)
            },
            Err(err) => Err(err),
        }
    }

    /// Idリストから位置を取得します。
    ///
    /// # Arguments
    ///
    /// * `id` - 取得するEntityのIdです。
    ///
    fn get_position(&self, id: &Id) -> Result<(usize, usize), Error> {

        if let Some(Some(info)) = self.ids.get(id.index()) {

            Ok(info.index())

        } else {

            Err(Error::NonExistentId(id.clone()))
        }
    }

    /// Idリストに位置を設定します。
    ///
    /// # Arguments
    ///
    /// * `id` - 設定するEntityのIdです。
    ///
    fn set_position(&mut self, id: &Id, chunk_idx: usize, datas_idx: usize) -> Result<(), Error> {

        if let Some(Some(info)) = self.ids.get_mut(id.index()) {

            info.chunk_idx = chunk_idx as u32;
            info.datas_idx = datas_idx as u32;
            Ok(())

        } else {

            Err(Error::NonExistentId(id.clone()))
        }
    }
    
    /// データを生成します。
    /// 
    /// # Arguments 
    /// 
    /// * `id` - データに紐づけるIdです。
    /// * `args` - コンポーネントデータの初期値です。
    /// 
    fn spawn_data(&mut self, id: &Id, args: &Datas) -> Result<(usize, usize), Error> {
        
        let chunk_idx = self.match_chunk_idx(&args.request());
        let data_idx = match self.chunks.get_mut(chunk_idx)
        .expect("重大なエラーが発生しました。ECS/spawn_data/0") // !このエラーが発生した場合、match_chunk_idxが機能していない場合があります。
        .attach(&id, &args) {
                    
            Ok(data_idx) => data_idx,
            Err(err) => {
                return Err(Error::ProcessError(err));
            },
        };
        
        Ok((chunk_idx, data_idx))
    }
    
    /// データを削除します。
    /// 
    /// # Arguments 
    /// 
    /// * `id` - データに紐づいているIdです。
    /// * `chunk_idx` - チャンクの位置です。
    /// * `datas_idx` - データの位置です。
    /// 
    fn despawn_data(&mut self, chunk_idx: usize, datas_idx: usize) -> Result<(), Error> {
                
        match self.chunks
        .get_mut(chunk_idx)
        .expect("重大なエラーが発生しました。ECS/despawn_data/0") // !このエラーが発生した場合、remove_idの戻り値が不正な場合があります。
        .dettach(datas_idx) {
                 
            Ok(_) => Ok(()),
            Err(err) => Err(Error::ProcessError(err)),
        }
    }

    /// データをコピーした後に削除します。
    /// 
    /// # Arguments 
    /// 
    /// * `id` - データに紐づいているIdです。
    /// * `chunk_idx` - チャンクの位置です。
    /// * `datas_idx` - データの位置です。
    /// 
    fn pop_data(&mut self, chunk_idx: usize, datas_idx: usize) -> Result<Datas, Error> {
              
        let datas = match self.chunks
        .get_mut(chunk_idx)
        .expect("重大なエラーが発生しました。ECS/pop_data/0") // !このエラーが発生した場合、chunk_idx、または、datas_idxが不正な可能性があります。
        .datas(datas_idx) {
                 
            Ok(datas) => datas,
            Err(err) => return Err(Error::ProcessError(err)),
        };

        match self.chunks
        .get_mut(chunk_idx)
        .expect("重大なエラーが発生しました。ECS/pop_data/1") // !このエラーが発生した場合、chunk_idx、または、datas_idxが不正な可能性があります。
        .dettach(datas_idx) {
                 
            Ok(_) => Ok(datas),
            Err(err) => Err(Error::ProcessError(err)),
        }
    }
    
    /// リクエストと一致するチャンクの位置を取得します。
    /// ない場合は、新規に作成して、その位置を返します。
    /// 
    /// # Arguments 
    /// 
    /// * `req` - 判定するリクエストです。
    /// 
    fn match_chunk_idx(&mut self, req: &Request) -> usize {
        
        // チャンクを探します。
        for i in 0..self.chunks.len() {
            
            if self.chunks[i].match_req(req) {
                
                return i;
            }
        }
        
        // チャンクを生成します。
        let idx = self.chunks.len();
        self.chunks.push(Chunk::new(req));
        
        idx
    }
}

/// Idと関連データを紐づける情報です。
struct IdInfo {

    id: Id,
    chunk_idx: u32, // Idと紐づくデータのあるチャンクの位置。(Id<=u32::Max)
    datas_idx: u32, // Idと紐づくデータのある位置。(Id<=u32::Max)
}
impl IdInfo {

    /// 生成します。
    ///
    /// # Arguments
    ///
    /// * `id` - 結びつけるEntityのIdです。
    /// * `chunk_idx` - チャンクの位置です。
    /// * `datas_idx` - データの位置です。
    fn new(id: &Id, chunk_idx: usize, datas_idx: usize) -> Self {

        IdInfo { 
            id: id.clone(), 
            chunk_idx: chunk_idx as u32, 
            datas_idx: datas_idx as u32 
        }
    }
    
    /// chunk_idxとdatas_idxをタプルで取得します。
    fn index(&self) -> (usize, usize) {
        
        (self.chunk_idx as usize, self.datas_idx as usize)
    }
}

/// エラーです。
#[derive(Debug)]
pub enum Error {
    
    /// 同時に存在可能なIdの最大値を超過しました。
    IdOverflow, 
    /// 存在しないIdです。
    NonExistentId(Id),
    /// 内部処理に起因するエラーです。
    ProcessError(data::Error),
}
impl Display for Error {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        match self {

            Error::IdOverflow => f.write_str("同時に存在可能なIdの最大値を超過しました。"),
            Error::NonExistentId(id) => f.write_fmt(format_args!("Id: {} は存在しませんでした。", id.index())),
            Error::ProcessError(err) => f.write_fmt(format_args!("内部処理に起因するエラーです。>> {}", err.to_string())),
        }
    }
}
impl std::error::Error for Error {

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        
        match self {
            
            Error::ProcessError(err) => Some(err),
            _ => None
        }
    }
}
