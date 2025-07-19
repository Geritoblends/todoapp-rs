# Todo app backend built in Rust using sqlx, PostgreSQL and Docker

#### Tools needed

* `sqlx-cli`
Needed for the database schema
To use it you do `sqlx migrate add <migration-name>`. That generates a .sql file in the migrations/ folder, then  you edit that sql file and do `sqlx migrate run` to execute it.

#### Useful aliases

```{bash}
# Bash aliases
alias dcu="docker compose up -d"
alias dcd="docker compose down"
alias sqlclient="psql -h localhost -U postgres"
```
