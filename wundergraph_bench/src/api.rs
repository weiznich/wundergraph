#![cfg_attr(feature = "clippy", allow(similar_names))]
use chrono::NaiveDateTime;
use wundergraph::query_builder::types::{HasMany, HasOne};
use wundergraph::scalar::WundergraphScalarValue;

table! {
    actors (actor_id) {
        actor_id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        last_update -> Timestamp,
    }
}

table! {
    albums (album_id) {
        album_id -> Int4,
        title -> Varchar,
        artist_id -> Int4,
    }
}

table! {
    artists (artist_id) {
        artist_id -> Int4,
        name -> Nullable<Varchar>,
    }
}

table! {
    categories (category_id) {
        category_id -> Int4,
        name -> Varchar,
        last_update -> Timestamp,
    }
}

table! {
    customers (customer_id) {
        customer_id -> Int4,
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
    employees (employee_id) {
        employee_id -> Int4,
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

#[cfg(feature = "postgres")]
table! {
    films (film_id) {
        film_id -> Int4,
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

#[cfg(feature = "sqlite")]
table! {
    films (film_id) {
        film_id -> Int4,
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
//        special_features -> Nullable<Array<Text>>,
    //    fulltext -> Tsvector,
    }
}

table! {
    genres (genre_id) {
        genre_id -> Int4,
        name -> Nullable<Varchar>,
    }
}

table! {
    invoice_lines (invoice_line_id) {
        invoice_line_id -> Int4,
        invoice_id -> Int4,
        track_id -> Int4,
//        unit_price -> Numeric,
        quantity -> Int4,
    }
}

table! {
    invoices (invoice_id) {
        invoice_id -> Int4,
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
    media_types (media_type_id) {
        media_type_id -> Int4,
        name -> Nullable<Varchar>,
    }
}

table! {
    playlists (playlist_id) {
        playlist_id -> Int4,
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
    tracks (track_id) {
        track_id -> Int4,
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

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "actors"]
#[primary_key(actor_id)]
pub struct Actor {
    actor_id: i32,
    first_name: String,
    last_name: String,
    last_update: NaiveDateTime,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "albums"]
#[primary_key(album_id)]
pub struct Album {
    album_id: i32,
    title: String,
    artist_id: HasOne<i32, Artist>,
    tracks: HasMany<Track, tracks::album_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "artists"]
#[primary_key(artist_id)]
pub struct Artist {
    artist_id: i32,
    name: Option<String>,
    albums: HasMany<Album, albums::artist_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "categories"]
#[primary_key(category_id)]
pub struct Category {
    category_id: i32,
    name: String,
    last_update: NaiveDateTime,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "customers"]
#[primary_key(customer_id)]
pub struct Customer {
    customer_id: i32,
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
    support_rep_id: Option<HasOne<i32, Employe>>,
    invoices: HasMany<Invoice, invoices::customer_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "employees"]
#[primary_key(employee_id)]
pub struct Employe {
    employee_id: i32,
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
    customers: HasMany<Customer, customers::support_rep_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity, Copy)]
#[table_name = "film_actor"]
#[primary_key(actor_id, film_id)]
pub struct FilmActor {
    actor_id: i16,
    film_id: i16,
    last_update: NaiveDateTime,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "films"]
#[primary_key(film_id)]
pub struct Film {
    film_id: i32,
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
    #[cfg(feature = "postgres")]
    special_features: Option<Vec<String>>,
    //fulltext: Tsvector,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "genres"]
#[primary_key(genre_id)]
pub struct Genre {
    genre_id: i32,
    name: Option<String>,
    tracks: HasMany<Track, tracks::genre_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "invoice_lines"]
#[primary_key(invoice_line_id)]
pub struct InvoiceLine {
    invoice_line_id: i32,
    invoice_id: HasOne<i32, Invoice>,
    track_id: HasOne<i32, Track>,
    //    unit_price: BigDecimal,
    quantity: i32,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "invoices"]
#[primary_key(invoice_id)]
pub struct Invoice {
    invoice_id: i32,
    customer_id: HasOne<i32, Customer>,
    invoice_date: NaiveDateTime,
    billing_address: Option<String>,
    billing_city: Option<String>,
    billing_state: Option<String>,
    billing_country: Option<String>,
    billing_postal_code: Option<String>,
    //    total: BigDecimal,
    invoice_lines: HasMany<InvoiceLine, invoice_lines::invoice_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "media_types"]
#[primary_key(media_type_id)]
pub struct MediaType {
    media_type_id: i32,
    name: Option<String>,
    tracks: HasMany<Track, tracks::media_type_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "playlists"]
#[primary_key(playlist_id)]
pub struct Playlist {
    playlist_id: i32,
    name: Option<String>,
    playlist_track: HasMany<PlaylistTrack, playlist_track::playlist_id>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "playlist_track"]
#[primary_key(playlist_id, track_id)]
pub struct PlaylistTrack {
    playlist_id: HasOne<i32, Playlist>,
    track_id: HasOne<i32, Track>,
}

#[derive(Clone, Debug, Identifiable, WundergraphEntity)]
#[table_name = "tracks"]
#[primary_key(track_id)]
pub struct Track {
    track_id: i32,
    name: String,
    album_id: Option<HasOne<i32, Album>>,
    media_type_id: HasOne<i32, MediaType>,
    genre_id: Option<HasOne<i32, Genre>>,
    composer: Option<String>,
    milliseconds: i32,
    bytes: Option<i32>,
    //    unit_price: BigDecimal,
    invoice_lines: HasMany<InvoiceLine, invoice_lines::track_id>,
    playlist_track: HasMany<PlaylistTrack, playlist_track::track_id>,
}

wundergraph::query_object! {
    Query {
        Actor,
        Album,
        Artist,
        Category,
        Customer,
        Employe,
        FilmActor,
        Film,
        Genre,
        InvoiceLine,
        Invoice,
        MediaType,
        Playlist,
        Track,
    }
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "actors"]
pub struct NewActor {
    first_name: String,
    last_name: String,
    last_update: NaiveDateTime,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "actors"]
#[primary_key(actor_id)]
pub struct ActorChangeset {
    actor_id: i32,
    first_name: String,
    last_name: String,
    last_update: NaiveDateTime,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "albums"]
pub struct NewAlbum {
    title: String,
    artist_id: i32,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "albums"]
#[primary_key(album_id)]
pub struct AlbumChangeset {
    album_id: i32,
    title: String,
    artist_id: i32,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "artists"]
pub struct NewArtist {
    name: Option<String>,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "artists"]
#[primary_key(artist_id)]
pub struct ArtistChangeset {
    artist_id: i32,
    name: Option<String>,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "categories"]
pub struct NewCategorie {
    name: String,
    last_update: NaiveDateTime,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "categories"]
#[primary_key(category_id)]
pub struct CategorieChangeset {
    category_id: i32,
    name: String,
    last_update: NaiveDateTime,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "customers"]
pub struct NewCustomer {
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
#[primary_key(customer_id)]
pub struct CustomerChangeset {
    customer_id: i32,
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
pub struct NewEmployee {
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
#[primary_key(employee_id)]
pub struct EmployeeChangeset {
    employee_id: i32,
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
pub struct NewFilmActor {
    last_update: NaiveDateTime,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug, Copy)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "film_actor"]
#[primary_key(actor_id, film_id)]
pub struct FilmActorChangeset {
    actor_id: i16,
    film_id: i16,
    last_update: NaiveDateTime,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[graphql(scalar = "WundergraphScalarValue")]
#[table_name = "films"]
pub struct NewFilm {
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
    #[cfg(feature = "postgres")]
    special_features: Option<Vec<String>>,
    //fulltext: Tsvector,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "films"]
#[primary_key(film_id)]
#[graphql(scalar = "WundergraphScalarValue")]
pub struct FilmChangeset {
    film_id: i32,
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
    #[cfg(feature = "postgres")]
    special_features: Option<Vec<String>>,
    //    fulltext: Tsvector,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "genres"]
pub struct NewGenre {
    name: Option<String>,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "genres"]
#[primary_key(genre_id)]
pub struct GenreChangeset {
    genre_id: i32,
    name: Option<String>,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug, Copy)]
#[table_name = "invoice_lines"]
pub struct NewInvoiceLine {
    invoice_id: i32,
    track_id: i32,
    //    unit_price: BigDecimal,
    quantity: i32,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug, Copy)]
#[table_name = "invoice_lines"]
#[primary_key(invoice_line_id)]
pub struct InvoiceLineChangeset {
    invoice_line_id: i32,
    invoice_id: i32,
    track_id: i32,
    //    unit_price: BigDecimal,
    quantity: i32,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "invoices"]
pub struct NewInvoice {
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
#[primary_key(invoice_id)]
pub struct InvoiceChangeset {
    invoice_id: i32,
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
pub struct NewMediaType {
    name: Option<String>,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "media_types"]
#[primary_key(media_type_id)]
pub struct MediaTypeChangeset {
    media_type_id: i32,
    name: Option<String>,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "playlists"]
pub struct NewPlaylist {
    name: Option<String>,
}

#[derive(AsChangeset, Identifiable, GraphQLInputObject, Clone, Debug)]
#[table_name = "playlists"]
#[primary_key(playlist_id)]
pub struct PlaylistChangeset {
    playlist_id: i32,
    name: Option<String>,
}

#[derive(Insertable, GraphQLInputObject, Clone, Debug)]
#[table_name = "tracks"]
pub struct NewTrack {
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
#[primary_key(track_id)]
pub struct TrackChangeset {
    track_id: i32,
    name: String,
    album_id: Option<i32>,
    media_type_id: i32,
    genre_id: Option<i32>,
    composer: Option<String>,
    milliseconds: i32,
    bytes: Option<i32>,
    //    unit_price: BigDecimal,
}

wundergraph::mutation_object! {
    Mutation{
        Actor(insert = NewActor, update = ActorChangeset, ),
        Album(insert = NewAlbum, update = AlbumChangeset, ),
        Artist(insert = NewArtist, update = ArtistChangeset, ),
        Category(insert = NewCategorie, update = CategorieChangeset, ),
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
