#![cfg_attr(feature = "clippy", allow(similar_names))]
use chrono::NaiveDateTime;
use wundergraph::query_helper::{HasMany, HasOne};

table! {
    actors (id) {
        id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        last_update -> Timestamp,
    }
}

table! {
    albums (id) {
        id -> Int4,
        title -> Varchar,
        artist_id -> Int4,
    }
}

table! {
    artists (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
    }
}

table! {
    categories (id) {
        id -> Int4,
        name -> Varchar,
        last_update -> Timestamp,
    }
}

table! {
    customers (id) {
        id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        company -> Nullable<Varchar>,
        address -> Nullable<Varchar>,
        city -> Nullable<Varchar>,
        state -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        postal_code -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        fax -> Nullable<Varchar>,
        email -> Varchar,
        support_rep_id -> Nullable<Int4>,
    }
}

table! {
    employees (id) {
        id -> Int4,
        last_name -> Varchar,
        first_name -> Varchar,
        title -> Nullable<Varchar>,
        reports_to -> Nullable<Int4>,
        birth_date -> Nullable<Timestamp>,
        hire_date -> Nullable<Timestamp>,
        address -> Nullable<Varchar>,
        city -> Nullable<Varchar>,
        state -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        postal_code -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        fax -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
    }
}

table! {
    film_actor (actor_id, film_id) {
        actor_id -> Int2,
        film_id -> Int2,
        last_update -> Timestamp,
    }
}

table! {
    films (id) {
        id -> Int4,
        title -> Varchar,
        description -> Nullable<Text>,
        release_year -> Nullable<Int4>,
        language_id -> Int2,
        rental_duration -> Int2,
//        rental_rate -> Numeric,
        length -> Nullable<Int2>,
   //     replacement_cost -> Numeric,
        rating -> Nullable<Text>,
        last_update -> Timestamp,
        special_features -> Nullable<Array<Text>>,
    //    fulltext -> Tsvector,
    }
}

table! {
    genres (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
    }
}

table! {
    invoice_lines (id) {
        id -> Int4,
        invoice_id -> Int4,
        track_id -> Int4,
//        unit_price -> Numeric,
        quantity -> Int4,
    }
}

table! {
    invoices (id) {
        id -> Int4,
        customer_id -> Int4,
        invoice_date -> Timestamp,
        billing_address -> Nullable<Varchar>,
        billing_city -> Nullable<Varchar>,
        billing_state -> Nullable<Varchar>,
        billing_country -> Nullable<Varchar>,
        billing_postal_code -> Nullable<Varchar>,
//        total -> Numeric,
    }
}

table! {
    media_types (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
    }
}

table! {
    playlists (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
    }
}

table! {
    playlist_track (playlist_id, track_id) {
        playlist_id -> Int4,
        track_id -> Int4,
    }
}

table! {
    tracks (id) {
        id -> Int4,
        name -> Varchar,
        album_id -> Nullable<Int4>,
        media_type_id -> Int4,
        genre_id -> Nullable<Int4>,
        composer -> Nullable<Varchar>,
        milliseconds -> Int4,
        bytes -> Nullable<Int4>,
//        unit_price -> Numeric,
    }
}

allow_tables_to_appear_in_same_query!(
    actors,
    albums,
    artists,
    categories,
    customers,
    employees,
    film_actor,
    films,
    genres,
    invoice_lines,
    invoices,
    media_types,
    playlists,
    playlist_track,
    tracks,
);

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "actors"]
#[primary_key(id)]
struct Actor {
    id: i32,
    first_name: String,
    last_name: String,
    last_update: NaiveDateTime,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "albums"]
#[primary_key(id)]
#[belongs_to(Artist, foreign_key = "artist_id")]
struct Album {
    id: i32,
    title: String,
    artist_id: HasOne<i32, Artist>,
    #[wundergraph(is_nullable_reference = "true")]
    tracks: HasMany<Track>,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "artists"]
#[primary_key(id)]
struct Artist {
    id: i32,
    name: Option<String>,
    albums: HasMany<Album>,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "categories"]
#[primary_key(id)]
struct Categorie {
    id: i32,
    name: String,
    last_update: NaiveDateTime,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "customers"]
#[primary_key(id)]
#[belongs_to(Employee, foreign_key = "support_rep_id")]
struct Customer {
    id: i32,
    first_name: String,
    last_name: String,
    company: Option<String>,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    postal_code: Option<String>,
    phone: Option<String>,
    fax: Option<String>,
    email: String,
    support_rep_id: Option<HasOne<i32, Employee>>,
    invoices: HasMany<Invoice>,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "employees"]
#[primary_key(id)]
struct Employee {
    id: i32,
    last_name: String,
    first_name: String,
    title: Option<String>,
    reports_to: Option<i32>,
    birth_date: Option<NaiveDateTime>,
    hire_date: Option<NaiveDateTime>,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    postal_code: Option<String>,
    phone: Option<String>,
    fax: Option<String>,
    email: Option<String>,
    #[wundergraph(is_nullable_reference = "true")]
    customers: HasMany<Customer>,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations, Copy)]
#[table_name = "film_actor"]
#[primary_key(actor_id, film_id)]
struct FilmActor {
    actor_id: i16,
    film_id: i16,
    last_update: NaiveDateTime,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "films"]
#[primary_key(id)]
struct Film {
    id: i32,
    title: String,
    description: Option<String>,
    release_year: Option<i32>,
    language_id: i16,
    rental_duration: i16,
    //    rental_rate: BigDecimal,
    length: Option<i16>,
    //  replacement_cost: BigDecimal,
    rating: Option<String>,
    last_update: NaiveDateTime,
    special_features: Option<Vec<String>>,
    //fulltext: Tsvector,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "genres"]
#[primary_key(id)]
struct Genre {
    id: i32,
    name: Option<String>,
    #[wundergraph(is_nullable_reference = "true")]
    tracks: HasMany<Track>,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "invoice_lines"]
#[primary_key(id)]
#[belongs_to(Invoice, foreign_key = "invoice_id")]
#[belongs_to(Track, foreign_key = "track_id")]
struct InvoiceLine {
    id: i32,
    invoice_id: HasOne<i32, Invoice>,
    track_id: HasOne<i32, Track>,
    //    unit_price: BigDecimal,
    quantity: i32,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "invoices"]
#[primary_key(id)]
#[belongs_to(Customer, foreign_key = "customer_id")]
struct Invoice {
    id: i32,
    customer_id: HasOne<i32, Customer>,
    invoice_date: NaiveDateTime,
    billing_address: Option<String>,
    billing_city: Option<String>,
    billing_state: Option<String>,
    billing_country: Option<String>,
    billing_postal_code: Option<String>,
    //    total: BigDecimal,
    invoice_lines: HasMany<InvoiceLine>,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "media_types"]
#[primary_key(id)]
struct MediaType {
    id: i32,
    name: Option<String>,
    tracks: HasMany<Track>,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "playlists"]
#[primary_key(id)]
struct Playlist {
    id: i32,
    name: Option<String>,
    playlist_track: HasMany<PlaylistTrack>,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "playlist_track"]
#[primary_key(playlist_id, track_id)]
#[belongs_to(Playlist, foreign_key = "playlist_id")]
#[belongs_to(Track, foreign_key = "track_id")]
struct PlaylistTrack {
    playlist_id: HasOne<i32, Playlist>,
    track_id: HasOne<i32, Track>,
}

#[derive(Clone, Debug, Queryable, Eq, PartialEq, Hash, Identifiable, WundergraphEntity,
         WundergraphFilter, Associations)]
#[table_name = "tracks"]
#[primary_key(id)]
#[belongs_to(Album, foreign_key = "album_id")]
#[belongs_to(Genre, foreign_key = "genre_id")]
#[belongs_to(MediaType, foreign_key = "media_type_id")]
struct Track {
    id: i32,
    name: String,
    album_id: Option<HasOne<i32, Album>>,
    media_type_id: HasOne<i32, MediaType>,
    genre_id: Option<HasOne<i32, Genre>>,
    composer: Option<String>,
    milliseconds: i32,
    bytes: Option<i32>,
    //    unit_price: BigDecimal,
    invoice_lines: HasMany<InvoiceLine>,
    playlist_track: HasMany<PlaylistTrack>,
}

wundergraph_query_object!{
    Query {
        Actors(Actor, filter = ActorFilter),
        Albums(Album, filter = AlbumFilter),
        Artists(Artist, filter = ArtistFilter),
        Categories(Categorie, filter = CategorieFilter),
        Customers(Customer, filter = CustomerFilter),
        Employees(Employee, filter = EmployeeFilter),
        FilmActor(FilmActor, filter = FilmActorFilter),
        Films(Film, filter = FilmFilter),
        Genres(Genre, filter = GenreFilter),
        InvoiceLines(InvoiceLine, filter = InvoiceLineFilter),
        Invoices(Invoice, filter = InvoiceFilter),
        MediaTypes(MediaType, filter = MediaTypeFilter),
        Playlists(Playlist, filter = PlaylistFilter),
//        PlaylistTrack(PlaylistTrack, filter = PlaylistTrackFilter),
        Tracks(Track, filter = TrackFilter),
    }
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "actors"]
struct NewActor {
    first_name: String,
    last_name: String,
    last_update: NaiveDateTime,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "actors"]
#[primary_key(id)]
struct ActorChangeset {
    id: i32,
    first_name: String,
    last_name: String,
    last_update: NaiveDateTime,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "albums"]
struct NewAlbum {
    title: String,
    artist_id: i32,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "albums"]
#[primary_key(id)]
struct AlbumChangeset {
    id: i32,
    title: String,
    artist_id: i32,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "artists"]
struct NewArtist {
    name: Option<String>,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "artists"]
#[primary_key(id)]
struct ArtistChangeset {
    id: i32,
    name: Option<String>,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "categories"]
struct NewCategorie {
    name: String,
    last_update: NaiveDateTime,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "categories"]
#[primary_key(id)]
struct CategorieChangeset {
    id: i32,
    name: String,
    last_update: NaiveDateTime,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "customers"]
struct NewCustomer {
    first_name: String,
    last_name: String,
    company: Option<String>,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    postal_code: Option<String>,
    phone: Option<String>,
    fax: Option<String>,
    email: String,
    support_rep_id: Option<i32>,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "customers"]
#[primary_key(id)]
struct CustomerChangeset {
    id: i32,
    first_name: String,
    last_name: String,
    company: Option<String>,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    postal_code: Option<String>,
    phone: Option<String>,
    fax: Option<String>,
    email: String,
    support_rep_id: Option<i32>,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "employees"]
struct NewEmployee {
    last_name: String,
    first_name: String,
    title: Option<String>,
    reports_to: Option<i32>,
    birth_date: Option<NaiveDateTime>,
    hire_date: Option<NaiveDateTime>,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    postal_code: Option<String>,
    phone: Option<String>,
    fax: Option<String>,
    email: Option<String>,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "employees"]
#[primary_key(id)]
struct EmployeeChangeset {
    id: i32,
    last_name: String,
    first_name: String,
    title: Option<String>,
    reports_to: Option<i32>,
    birth_date: Option<NaiveDateTime>,
    hire_date: Option<NaiveDateTime>,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    postal_code: Option<String>,
    phone: Option<String>,
    fax: Option<String>,
    email: Option<String>,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug, Copy)]
#[table_name = "film_actor"]
struct NewFilmActor {
    last_update: NaiveDateTime,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug, Copy)]
#[table_name = "film_actor"]
#[primary_key(actor_id, film_id)]
struct FilmActorChangeset {
    actor_id: i16,
    film_id: i16,
    last_update: NaiveDateTime,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "films"]
struct NewFilm {
    title: String,
    description: Option<String>,
    release_year: Option<i32>,
    language_id: i16,
    rental_duration: i16,
    //    rental_rate: BigDecimal,
    length: Option<i16>,
    //  replacement_cost: BigDecimal,
    rating: Option<String>,
    last_update: NaiveDateTime,
    special_features: Option<Vec<String>>,
    //fulltext: Tsvector,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "films"]
#[primary_key(id)]
struct FilmChangeset {
    id: i32,
    title: String,
    description: Option<String>,
    release_year: Option<i32>,
    language_id: i16,
    rental_duration: i16,
    //    rental_rate: BigDecimal,
    length: Option<i16>,
    //  replacement_cost: BigDecimal,
    rating: Option<String>,
    last_update: NaiveDateTime,
    special_features: Option<Vec<String>>,
    //    fulltext: Tsvector,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "genres"]
struct NewGenre {
    name: Option<String>,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "genres"]
#[primary_key(id)]
struct GenreChangeset {
    id: i32,
    name: Option<String>,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug, Copy)]
#[table_name = "invoice_lines"]
struct NewInvoiceLine {
    invoice_id: i32,
    track_id: i32,
    //    unit_price: BigDecimal,
    quantity: i32,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug, Copy)]
#[table_name = "invoice_lines"]
#[primary_key(id)]
struct InvoiceLineChangeset {
    id: i32,
    invoice_id: i32,
    track_id: i32,
    //    unit_price: BigDecimal,
    quantity: i32,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "invoices"]
struct NewInvoice {
    customer_id: i32,
    invoice_date: NaiveDateTime,
    billing_address: Option<String>,
    billing_city: Option<String>,
    billing_state: Option<String>,
    billing_country: Option<String>,
    billing_postal_code: Option<String>,
    //    total: BigDecimal,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "invoices"]
#[primary_key(id)]
struct InvoiceChangeset {
    id: i32,
    customer_id: i32,
    invoice_date: NaiveDateTime,
    billing_address: Option<String>,
    billing_city: Option<String>,
    billing_state: Option<String>,
    billing_country: Option<String>,
    billing_postal_code: Option<String>,
    //    total: BigDecimal,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "media_types"]
struct NewMediaType {
    name: Option<String>,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "media_types"]
#[primary_key(id)]
struct MediaTypeChangeset {
    id: i32,
    name: Option<String>,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "playlists"]
struct NewPlaylist {
    name: Option<String>,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "playlists"]
#[primary_key(id)]
struct PlaylistChangeset {
    id: i32,
    name: Option<String>,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "tracks"]
struct NewTrack {
    name: String,
    album_id: Option<i32>,
    media_type_id: i32,
    genre_id: Option<i32>,
    composer: Option<String>,
    milliseconds: i32,
    bytes: Option<i32>,
    //    unit_price: BigDecimal,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "tracks"]
#[primary_key(id)]
struct TrackChangeset {
    id: i32,
    name: String,
    album_id: Option<i32>,
    media_type_id: i32,
    genre_id: Option<i32>,
    composer: Option<String>,
    milliseconds: i32,
    bytes: Option<i32>,
    //    unit_price: BigDecimal,
}

wundergraph_mutation_object!{
    Mutation{
        Actor(insert = NewActor, update = ActorChangeset, ),
        Album(insert = NewAlbum, update = AlbumChangeset, ),
        Artist(insert = NewArtist, update = ArtistChangeset, ),
        Categorie(insert = NewCategorie, update = CategorieChangeset, ),
        Customer(insert = NewCustomer, update = CustomerChangeset, ),
//        Employee(insert = NewEmployee, update = EmployeeChangeset, ),
        FilmActor(insert = NewFilmActor, update = FilmActorChangeset, ),
        Film(insert = NewFilm, update = FilmChangeset, ),
        Genre(insert = NewGenre, update = GenreChangeset, ),
        InvoiceLine(insert = NewInvoiceLine, update = InvoiceLineChangeset, ),
        Invoice(insert = NewInvoice, update = InvoiceChangeset, ),
        MediaType(insert = NewMediaType, update = MediaTypeChangeset, ),
        Playlist(insert = NewPlaylist, update = PlaylistChangeset, ),
   //     PlaylistTrack(),
        Track(insert = NewTrack, update = TrackChangeset, ),
    }
}
