CREATE TABLE IF NOT EXISTS properties (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    price BIGINT,
    address TEXT NOT NULL,
    province TEXT NOT NULL,
    city TEXT NOT NULL,
    suburb TEXT,
    property_type TEXT NOT NULL,
    bedrooms INTEGER,
    bathrooms INTEGER,
    garage_spaces INTEGER,
    land_size DOUBLE PRECISION,
    floor_size DOUBLE PRECISION,
    source_url TEXT NOT NULL,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION
);