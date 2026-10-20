set shell := ["bash", "-euo", "pipefail", "-c"]
set positional-arguments := true

root := justfile_directory()
worker_dir := root / "infra" / "worker"
database := "jayjay_stats"

list:
    @echo "worker recipes:"
    @echo "  just worker::build            Build the Rust Worker locally"
    @echo "  just worker::test             Run Rust and local Worker/D1 tests"
    @echo "  just worker::check-migrations Verify migration names and immutable checksums"
    @echo "  just worker::apply-schema     Apply pending migrations to remote D1"
    @echo "  just worker::deploy           Verify, migrate, verify remote D1, then deploy"
    @echo "  just worker::dau [days]       DAU and ping counts, default 30 days"
    @echo "  just worker::mau [months]     Monthly active installs, default 6 months"
    @echo "  just worker::versions [limit] Current-month version/build adoption, default top 20"
    @echo "  just worker::platforms        Current-month platform split"
    @echo "  just worker::systems [limit]  Current-month OS/arch split, default top 30"
    @echo "  just worker::recent [limit]   Recent telemetry rows, default 20"

build:
    cd "{{ worker_dir }}" && worker-build --release -- --locked

test: check-migrations
    cargo test -p jayjay-worker --locked -- --include-ignored

check-migrations:
    cd "{{ worker_dir }}" && python3 scripts/verify_migrations.py

apply-schema: check-migrations
    cd "{{ worker_dir }}" && wrangler d1 migrations apply "{{ database }}" --remote

deploy: apply-schema
    cd "{{ worker_dir }}" && python3 scripts/verify_migrations.py --remote --database "{{ database }}"
    cd "{{ worker_dir }}" && wrangler deploy

dau days='30':
    bash "{{root}}/scripts/worker-query.sh" "{{worker_dir}}" "{{database}}" dau "$1"

mau months='6':
    bash "{{root}}/scripts/worker-query.sh" "{{worker_dir}}" "{{database}}" mau "$1"

versions limit='20':
    bash "{{root}}/scripts/worker-query.sh" "{{worker_dir}}" "{{database}}" versions "$1"

platforms:
    bash "{{root}}/scripts/worker-query.sh" "{{worker_dir}}" "{{database}}" platforms

systems limit='30':
    bash "{{root}}/scripts/worker-query.sh" "{{worker_dir}}" "{{database}}" systems "$1"

recent limit='20':
    bash "{{root}}/scripts/worker-query.sh" "{{worker_dir}}" "{{database}}" recent "$1"
