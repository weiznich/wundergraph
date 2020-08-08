CREATE TABLE species(
    id INT AUTO_INCREMENT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE home_worlds(
   id INT AUTO_INCREMENT PRIMARY KEY,
   name TEXT NOT NULL
);

CREATE TABLE heros(
    id INT AUTO_INCREMENT PRIMARY KEY,
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
    hero_id INTEGER NOT NULL REFERENCES heros(id) ON DELETE CASCADE ON UPDATE RESTRICT,
    friend_id INTEGER NOT NULL REFERENCES heros(id) ON DELETE CASCADE ON UPDATE RESTRICT,
    PRIMARY KEY(hero_id, friend_id)
);

INSERT INTO species(id, name) VALUES (1, 'Human'), (2, 'Robot');

INSERT INTO home_worlds(id, name) VALUES(1, 'Tatooine'), (2, 'Alderaan');

INSERT INTO heros(id, name, species, home_world, hair_color)
    VALUES (1, 'Luke Skywalker', 1, 1, 'blond'),
           (2, 'Darth Vader', 1, 1, DEFAULT),
           (3, 'Han Solo', 1, Null, DEFAULT),
           (4, 'Leia Organa', 1, 2, DEFAULT),
           (5, 'Wilhuff Tarkin', 1, Null, DEFAULT);

INSERT INTO appears_in(hero_id, episode)
    VALUES (1, 1), (1, 2), (1, 3),
           (2, 1), (2, 2), (2, 3),
           (3, 1), (3, 2), (3, 3),
           (4, 1), (4, 2), (4, 3),
           (5, 3);


INSERT INTO friends(hero_id, friend_id)
    VALUES (1, 3), (1, 4), (2, 5), (3, 1),
           (3, 4), (4, 1), (4, 3), (5, 2);
