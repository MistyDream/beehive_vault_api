# Database migrations

SQLx migrations live in this directory and use names such as
`202608100001_create_households.sql`.

The application applies pending migrations during startup. A migration must be
reviewed as an immutable part of the database history once it has been shared.
