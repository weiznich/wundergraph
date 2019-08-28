-- Your SQL goes here

CREATE TABLE species(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

CREATE TABLE home_worlds(
   id INTEGER PRIMARY KEY AUTOINCREMENT,
   name TEXT NOT NULL
);

CREATE TABLE heros(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    hair_color TEXT,
    species INTEGER NOT NULL REFERENCES species(id) ON DELETE CASCADE ON UPDATE RESTRICT,
    home_world INTEGER REFERENCES home_worlds(id) ON DELETE CASCADE ON UPDATE RESTRICT
);

CREATE TABLE appears_in(
    hero_id INTEGER NOT NULL REFERENCES heros(id) ON DELETE CASCADE ON UPDATE RESTRICT,
    episode SMALLINT NOT NULL CHECK(episode IN (1,2,3)),
    PRIMARY KEY(hero_id, episode)
);

CREATE TABLE friends(
    hero_id INTEGER NOT NULL REFERENCES friends(id) ON DELETE CASCADE ON UPDATE RESTRICT,
    friend_id INTEGER NOT NULL REFERENCES friends(id) ON DELETE CASCADE ON UPDATE RESTRICT,
    PRIMARY KEY(hero_id, friend_id)
);

INSERT INTO species(name) VALUES('Human');
INSERT INTO species(name) VALUES('Robot');

INSERT INTO home_worlds(name) VALUES('Tatooine');
INSERT INTO home_worlds(name) VALUES('Alderaan');

INSERT INTO heros(name, species, home_world, hair_color) VALUES ('Luke Skywalker', 1, 1, 'blond');
INSERT INTO heros(name, species, home_world) VALUES ('Darth Vader', 1, 1);
INSERT INTO heros(name, species, home_world) VALUES ('Han Solo', 1, Null);
INSERT INTO heros(name, species, home_world) VALUES ('Leia Organa', 1, 2);
INSERT INTO heros(name, species, home_world) VALUES ('Wilhuff Tarkin', 1, Null);

INSERT INTO appears_in(hero_id, episode) VALUES(1, 1);
INSERT INTO appears_in(hero_id, episode) VALUES(1, 2);
INSERT INTO appears_in(hero_id, episode) VALUES(1, 3);

INSERT INTO appears_in(hero_id, episode) VALUES(2, 1);
INSERT INTO appears_in(hero_id, episode) VALUES(2, 2);
INSERT INTO appears_in(hero_id, episode) VALUES(2, 3);

INSERT INTO appears_in(hero_id, episode) VALUES(3, 1);
INSERT INTO appears_in(hero_id, episode) VALUES(3, 2);
INSERT INTO appears_in(hero_id, episode) VALUES(3, 3);

INSERT INTO appears_in(hero_id, episode) VALUES(4, 1);
INSERT INTO appears_in(hero_id, episode) VALUES(4, 2);
INSERT INTO appears_in(hero_id, episode) VALUES(4, 3);

INSERT INTO appears_in(hero_id, episode) VALUES(5, 3);

INSERT INTO friends(hero_id, friend_id) VALUES(1, 3);
INSERT INTO friends(hero_id, friend_id) VALUES(1, 4);

INSERT INTO friends(hero_id, friend_id) VALUES(2, 5);

INSERT INTO friends(hero_id, friend_id) VALUES(3, 1);
INSERT INTO friends(hero_id, friend_id) VALUES(3, 4);

INSERT INTO friends(hero_id, friend_id) VALUES(4, 1);
INSERT INTO friends(hero_id, friend_id) VALUES(4, 3);

INSERT INTO friends(hero_id, friend_id) VALUES(5, 2);
