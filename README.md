# commune - ActivityPub backend server of Commune Project.

Commune is an ActivityPub-enabled forum software in its very very very early stage of development.

DO NOT use it in a production environment, NO function is done.

# Dependencies
* Rust (tested with rustc 1.50.0 (cb75ad5db 2021-02-10) stable-x86_64-unknown-linux-gnu)
* PostgreSQL (tested with 13.1-1.pgdg100+1)

# Development

You must prepare a PostgreSQL database for it.

Copy .env.sample to .env, and configure DATABASE_URL, LOCAL_DOMAINS and so on.

```bash
cargo install diesel_cli --no-default-features --features postgres
diesel database setup
cargo run --bin communectl
cargo run
```

And you can access the unfinished page on http://localhost:8000. You may need to set a proper reverse proxy before it.

# Run Unit Tests
#TODO

# License

Copyright (C) 2021 Misaka 0x4e21

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see https://www.gnu.org/licenses/.