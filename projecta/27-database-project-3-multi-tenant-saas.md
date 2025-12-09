# Chapter 27: Database Patterns

## Project 3: Multi-Tenant SaaS Database with Diesel ORM

### Problem Statement

Build a multi-tenant SaaS application database layer where each organization (tenant) has completely isolated data. Your system must:
- **Tenant isolation**: Organization A cannot access Organization B's data under any circumstances
- **Type-safe queries**: All database operations verified at compile time with Diesel's DSL
- **Schema migrations**: Version-controlled schema changes across development/staging/production
- **Concurrent editing**: Multiple users editing same resource, detect conflicts with optimistic locking
- **Performance**: Connection pooling, efficient queries, proper indexing
- **Production-ready**: Seeding, backups, rollback capabilities

This project demonstrates Diesel's **strongest type safety** (stronger than SQLx)—queries are pure Rust code, no SQL strings at all.

### Why It Matters

**Real-World Impact**: Multi-tenancy is fundamental to SaaS business models:
- **Salesforce**: 150K+ organizations (tenants) on shared infrastructure, complete data isolation
- **Slack**: Millions of workspaces (tenants), each workspace's data strictly separated
- **GitHub**: Organizations as tenants, repository data isolated per org
- **Shopify**: Stores as tenants, 2M+ stores on shared platform

**Performance Numbers**:
- **Shared database with tenant_id**: 1 database serves 100K+ tenants (cost-efficient)
- **Row-level security**: Add `WHERE tenant_id = $1` to every query (critical for security)
- **vs Separate databases**: Multi-tenant = 1 DB connection pool, separate DBs = 100K connection pools
- **Diesel compile-time guarantees**: Zero runtime SQL errors, refactoring safe

**Rust-Specific Advantage**: Diesel's type system prevents entire classes of bugs:
```rust
// Python/Django - forgetting tenant filter = security breach
users = User.objects.all()  # ❌ Returns ALL tenants' users!

// Diesel - impossible to forget tenant filter (type system enforces it)
users.filter(tenant_id.eq(current_tenant))  // ✅ Compiler ensures tenant filtering
```

### Use Cases

**When you need this pattern**:
1. **B2B SaaS platforms** - CRM (Salesforce), project management (Asana), helpdesk (Zendesk)
2. **Multi-tenant marketplaces** - E-commerce (Shopify), food delivery (restaurant dashboards)
3. **Enterprise software** - HR systems, ERP, document management (each company isolated)
4. **Team collaboration tools** - Slack, Microsoft Teams, Notion (workspace = tenant)
5. **Development platforms** - GitHub (orgs), GitLab (groups), CI/CD (per-company)
6. **White-label solutions** - Resellers run branded versions, data isolated per client

**Real Examples**:
- **Jira**: Projects belong to organizations, strict tenant separation
- **HubSpot**: CRM data partitioned by customer (portal)
- **Stripe**: Platform accounts, each has isolated data
- **Auth0**: Tenants for B2B auth, complete data isolation

### Learning Goals

- Master Diesel's type-safe query DSL (no SQL strings, pure Rust)
- Understand multi-tenant architecture patterns (tenant_id filtering)
- Learn schema migrations with diesel CLI (up/down migrations)
- Practice optimistic locking for concurrent edits (version column)
- Build production patterns (connection pooling, seeding, backups)
- Experience strongest compile-time guarantees (refactoring safety)

---

## Milestone 1: Diesel Setup and Basic CRUD with Type-Safe DSL

### Introduction

**Starting Point**: Set up Diesel, generate schema, and learn the type-safe query DSL.

**What We're Building**: A single tenant system with:
- Organizations table (tenants)
- Users table (belonging to organizations)
- Pure Rust queries (no SQL strings)
- Auto-generated `schema.rs` providing type safety

**Key Feature**: Unlike SQLx (SQL in macros), Diesel queries are pure Rust:
```rust
// Diesel - pure Rust, maximum type safety
users::table
    .filter(users::email.eq("alice@example.com"))
    .first::<User>(conn)?

// SQLx - SQL in string (still verified, but less Rust-native)
sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", "alice@example.com")
```

### Key Concepts

**Database Schema**:
```sql
CREATE TABLE organizations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    organization_id INT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    username VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, email)
);

CREATE INDEX idx_users_org ON users(organization_id);
```

**Generated Schema (diesel CLI)**:
```rust
// src/schema.rs (auto-generated)
diesel::table! {
    organizations (id) {
        id -> Int4,
        name -> Varchar,
        slug -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        organization_id -> Int4,
        email -> Varchar,
        username -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(users -> organizations (organization_id));
diesel::allow_tables_to_appear_in_same_query!(organizations, users);
```

**Structs/Types**:
```rust
#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = organizations)]
struct Organization {
    id: i32,
    name: String,
    slug: String,
    created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = organizations)]
struct NewOrganization<'a> {
    name: &'a str,
    slug: &'a str,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = users)]
struct User {
    id: i32,
    organization_id: i32,
    email: String,
    username: String,
    created_at: chrono::NaiveDateTime,
}
```

**Diesel Setup Commands**:
```bash
# Install diesel CLI
cargo install diesel_cli --no-default-features --features postgres

# Setup database (creates migrations directory)
diesel setup

# Create migration
diesel migration generate create_organizations

# Run migrations
diesel migration run

# Rollback (undo last migration)
diesel migration revert
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use diesel::prelude::*;

    fn establish_connection() -> PgConnection {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        PgConnection::establish(&database_url)
            .expect("Error connecting to database")
    }

    #[test]
    fn test_create_organization() {
        let mut conn = establish_connection();

        let org = create_organization(&mut conn, "Acme Corp", "acme").unwrap();

        assert_eq!(org.name, "Acme Corp");
        assert_eq!(org.slug, "acme");
        assert!(org.id > 0);
    }

    #[test]
    fn test_create_user() {
        let mut conn = establish_connection();

        let org = create_organization(&mut conn, "TestOrg", "testorg").unwrap();
        let user = create_user(&mut conn, org.id, "alice@example.com", "alice").unwrap();

        assert_eq!(user.organization_id, org.id);
        assert_eq!(user.email, "alice@example.com");
        assert_eq!(user.username, "alice");
    }

    #[test]
    fn test_get_users_by_organization() {
        let mut conn = establish_connection();

        let org = create_organization(&mut conn, "MyOrg", "myorg").unwrap();

        // Create multiple users
        create_user(&mut conn, org.id, "user1@example.com", "user1").unwrap();
        create_user(&mut conn, org.id, "user2@example.com", "user2").unwrap();

        let users = get_users_by_org(&mut conn, org.id).unwrap();

        assert_eq!(users.len(), 2);
        assert_eq!(users[0].organization_id, org.id);
        assert_eq!(users[1].organization_id, org.id);
    }

    #[test]
    fn test_unique_constraint() {
        let mut conn = establish_connection();

        let org = create_organization(&mut conn, "UniqueTest", "uniquetest").unwrap();

        create_user(&mut conn, org.id, "test@example.com", "test").unwrap();

        // Try to create duplicate email in same org
        let result = create_user(&mut conn, org.id, "test@example.com", "test2");
        assert!(result.is_err()); // Should fail
    }

    #[test]
    fn test_cascade_delete() {
        let mut conn = establish_connection();

        let org = create_organization(&mut conn, "TempOrg", "temporg").unwrap();
        create_user(&mut conn, org.id, "temp@example.com", "temp").unwrap();

        // Delete organization
        delete_organization(&mut conn, org.id).unwrap();

        // Users should be cascade deleted
        let users = get_users_by_org(&mut conn, org.id).unwrap();
        assert_eq!(users.len(), 0);
    }
}
```

### Starter Code

```rust
use diesel::prelude::*;
use diesel::pg::PgConnection;
use chrono::NaiveDateTime;

// Import generated schema
mod schema {
    include!("schema.rs");
}

use schema::{organizations, users};

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = organizations)]
struct Organization {
    id: i32,
    name: String,
    slug: String,
    created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = organizations)]
struct NewOrganization<'a> {
    name: &'a str,
    slug: &'a str,
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = users)]
struct User {
    id: i32,
    organization_id: i32,
    email: String,
    username: String,
    created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
struct NewUser<'a> {
    organization_id: i32,
    email: &'a str,
    username: &'a str,
}

/// Create organization
fn create_organization(
    conn: &mut PgConnection,
    name: &str,
    slug: &str,
) -> QueryResult<Organization> {
    // TODO: Use Diesel's DSL to INSERT
    // diesel::insert_into(organizations::table)
    //     .values(&NewOrganization { name, slug })
    //     .returning(Organization::as_returning())
    //     .get_result(conn)

    todo!()
}

/// Create user
fn create_user(
    conn: &mut PgConnection,
    organization_id: i32,
    email: &str,
    username: &str,
) -> QueryResult<User> {
    // TODO: INSERT user with Diesel DSL
    let new_user = NewUser {
        organization_id,
        email,
        username,
    };

    todo!(); // diesel::insert_into(users::table).values(&new_user).returning(User::as_returning()).get_result(conn)
}

/// Get users by organization
fn get_users_by_org(
    conn: &mut PgConnection,
    org_id: i32,
) -> QueryResult<Vec<User>> {
    // TODO: SELECT users WHERE organization_id = org_id
    // Notice: Pure Rust, no SQL strings!
    todo!(); // users::table.filter(users::organization_id.eq(org_id)).select(User::as_select()).load(conn)
}

/// Delete organization
fn delete_organization(
    conn: &mut PgConnection,
    org_id: i32,
) -> QueryResult<usize> {
    // TODO: DELETE organization (cascade deletes users)
    todo!(); // diesel::delete(organizations::table.filter(organizations::id.eq(org_id))).execute(conn)
}

/// Establish database connection
fn establish_connection(database_url: &str) -> PgConnection {
    PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn main() {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let mut conn = establish_connection(&database_url);

    // Example usage
    let org = create_organization(&mut conn, "Example Inc", "example").unwrap();
    println!("Created organization: {:?}", org);

    let user = create_user(&mut conn, org.id, "admin@example.com", "admin").unwrap();
    println!("Created user: {:?}", user);
}
```

### Check Your Understanding

- **What happens if you typo `users::emial` instead of `users::email`?** Won't compile! `schema.rs` defines available columns.
- **How is this different from SQLx?** Diesel = pure Rust DSL, SQLx = SQL in macros. Both type-safe, Diesel more Rust-native.
- **What generates `schema.rs`?** `diesel print-schema` (or automatic via `diesel.toml` config).
- **Why use `returning()` in INSERT?** PostgreSQL extension to get inserted row without separate SELECT.

---

## Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Critical Security Gap**: No tenant isolation! Current code can query ANY organization's users:
```rust
// ❌ BAD: Returns ALL users across ALL organizations
users::table.load::<User>(conn)?

// ✅ GOOD: Must filter by tenant
users::table.filter(users::organization_id.eq(current_tenant)).load(conn)?
```

**What We're Adding**:
- **Tenant-scoped queries**: All queries filtered by `organization_id`
- **Middleware pattern**: Inject current tenant context
- **Type-safe filtering**: Diesel ensures tenant filter applied

**Improvement**:
- **Security**: Impossible to accidentally query wrong tenant's data
- **Correctness**: Compiler enforces tenant isolation
- **Production-ready**: Standard multi-tenant pattern

---

## Milestone 2: Multi-Tenant Architecture (Tenant Isolation)

### Introduction

**The Problem**: Without explicit filtering, queries return data across all tenants (security breach).

**The Solution**:
1. Add `organization_id` to every resource table
2. **Always** filter by current tenant in queries
3. Use helper structs to enforce tenant context

**Pattern**:
```rust
// Tenant context struct
struct TenantContext {
    organization_id: i32,
}

// All queries scoped to tenant
impl TenantContext {
    fn get_users(&self, conn: &mut PgConnection) -> QueryResult<Vec<User>> {
        users::table
            .filter(users::organization_id.eq(self.organization_id))
            .load(conn)
    }
}
```

### Key Concepts

**Extended Schema** (add tenant_id everywhere):
```sql
CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    organization_id INT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE tasks (
    id SERIAL PRIMARY KEY,
    organization_id INT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    project_id INT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Ensure tenant consistency
CREATE INDEX idx_projects_org ON projects(organization_id);
CREATE INDEX idx_tasks_org ON tasks(organization_id);
```

**Tenant Context Pattern**:
```rust
struct TenantContext {
    organization_id: i32,
}

impl TenantContext {
    fn new(org_id: i32) -> Self {
        TenantContext { organization_id: org_id }
    }

    // All queries scoped to this tenant
    fn get_users(&self, conn: &mut PgConnection) -> QueryResult<Vec<User>> {
        users::table
            .filter(users::organization_id.eq(self.organization_id))
            .load(conn)
    }

    fn create_project(&self, conn: &mut PgConnection, name: &str, description: &str) -> QueryResult<Project> {
        // Automatically includes organization_id
        let new_project = NewProject {
            organization_id: self.organization_id,
            name,
            description,
        };

        diesel::insert_into(projects::table)
            .values(&new_project)
            .returning(Project::as_returning())
            .get_result(conn)
    }
}
```

### Checkpoint Tests

```rust
#[test]
fn test_tenant_isolation() {
    let mut conn = establish_connection();

    let org1 = create_organization(&mut conn, "Org1", "org1").unwrap();
    let org2 = create_organization(&mut conn, "Org2", "org2").unwrap();

    let ctx1 = TenantContext::new(org1.id);
    let ctx2 = TenantContext::new(org2.id);

    // Create users in different orgs
    ctx1.create_user(&mut conn, "user1@org1.com", "user1").unwrap();
    ctx2.create_user(&mut conn, "user1@org2.com", "user1").unwrap();

    // Each tenant sees only their users
    let org1_users = ctx1.get_users(&mut conn).unwrap();
    let org2_users = ctx2.get_users(&mut conn).unwrap();

    assert_eq!(org1_users.len(), 1);
    assert_eq!(org2_users.len(), 1);
    assert_ne!(org1_users[0].id, org2_users[0].id);
}

#[test]
fn test_cross_tenant_isolation() {
    let mut conn = establish_connection();

    let org1 = create_organization(&mut conn, "Org1", "org1").unwrap();
    let org2 = create_organization(&mut conn, "Org2", "org2").unwrap();

    let ctx1 = TenantContext::new(org1.id);
    let ctx2 = TenantContext::new(org2.id);

    // Org1 creates project
    let project = ctx1.create_project(&mut conn, "Secret Project", "Top secret").unwrap();

    // Org2 tries to access Org1's project
    let org2_projects = ctx2.get_projects(&mut conn).unwrap();

    assert_eq!(org2_projects.len(), 0); // Can't see Org1's projects
}

#[test]
fn test_cascading_tenant_filtering() {
    let mut conn = establish_connection();

    let org1 = create_organization(&mut conn, "Org1", "org1").unwrap();
    let ctx1 = TenantContext::new(org1.id);

    // Create project and tasks
    let project = ctx1.create_project(&mut conn, "Project", "Description").unwrap();
    ctx1.create_task(&mut conn, project.id, "Task 1").unwrap();
    ctx1.create_task(&mut conn, project.id, "Task 2").unwrap();

    // Get tasks (should be scoped to tenant)
    let tasks = ctx1.get_tasks(&mut conn).unwrap();
    assert_eq!(tasks.len(), 2);

    // All tasks belong to tenant
    for task in &tasks {
        assert_eq!(task.organization_id, org1.id);
    }
}
```

### Starter Code

```rust
#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = projects)]
struct Project {
    id: i32,
    organization_id: i32,
    name: String,
    description: Option<String>,
    created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = projects)]
struct NewProject<'a> {
    organization_id: i32,
    name: &'a str,
    description: &'a str,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = tasks)]
struct Task {
    id: i32,
    organization_id: i32,
    project_id: i32,
    title: String,
    completed: bool,
    created_at: NaiveDateTime,
}

/// Tenant context for scoped queries
struct TenantContext {
    organization_id: i32,
}

impl TenantContext {
    fn new(org_id: i32) -> Self {
        TenantContext { organization_id: org_id }
    }

    /// Get users (scoped to tenant)
    fn get_users(&self, conn: &mut PgConnection) -> QueryResult<Vec<User>> {
        // TODO: Filter by organization_id
        todo!(); // users::table.filter(users::organization_id.eq(self.organization_id)).load(conn)
    }

    /// Create user (automatically scoped to tenant)
    fn create_user(&self, conn: &mut PgConnection, email: &str, username: &str) -> QueryResult<User> {
        // TODO: INSERT with organization_id
        let new_user = NewUser {
            organization_id: self.organization_id,
            email,
            username,
        };

        todo!(); // diesel::insert_into(users::table).values(&new_user).returning(User::as_returning()).get_result(conn)
    }

    /// Create project (scoped to tenant)
    fn create_project(&self, conn: &mut PgConnection, name: &str, description: &str) -> QueryResult<Project> {
        // TODO: INSERT project with organization_id
        todo!();
    }

    /// Get projects (scoped to tenant)
    fn get_projects(&self, conn: &mut PgConnection) -> QueryResult<Vec<Project>> {
        // TODO: Filter by organization_id
        todo!();
    }

    /// Create task (scoped to tenant)
    fn create_task(&self, conn: &mut PgConnection, project_id: i32, title: &str) -> QueryResult<Task> {
        // TODO: INSERT task with organization_id
        // IMPORTANT: Verify project_id belongs to this tenant!
        let project = projects::table
            .filter(projects::id.eq(project_id))
            .filter(projects::organization_id.eq(self.organization_id))
            .first::<Project>(conn)?;

        todo!(); // Insert task
    }

    /// Get tasks (scoped to tenant)
    fn get_tasks(&self, conn: &mut PgConnection) -> QueryResult<Vec<Task>> {
        // TODO: Filter by organization_id
        todo!();
    }
}
```

---

## Why Milestone 2 Isn't Enough → Moving to Milestone 3

**Missing Features**:
1. **No complex queries**: Can't join across tables, filter by multiple conditions
2. **No sorting/pagination**: Real apps need ordered, paginated results
3. **No eager loading**: N+1 query problem (fetch project, then fetch tasks in loop)

**What We're Adding**:
- **JOINs with type safety**: Diesel's join syntax
- **Chained filters**: Multiple WHERE conditions
- **Sorting**: ORDER BY clauses
- **Eager loading**: Load related data in single query

**Improvement**:
- **Performance**: JOINs avoid N+1 queries
- **Expressiveness**: Complex queries still type-safe
- **Diesel's strength**: Complicated JOINs feel natural

---

## Milestone 3: Complex Queries with Diesel DSL

### Introduction

**The Problem**: Real apps need complex queries:
- Get projects with task count
- Filter by multiple conditions (completed tasks, recent projects)
- Sort and paginate results

**The Solution**: Diesel's query DSL supports:
- `inner_join()`, `left_join()`
- `filter()` chains for multiple conditions
- `order_by()` for sorting
- `limit()` and `offset()` for pagination

### Key Concepts

**JOIN Syntax**:
```rust
// Inner join
let results = projects::table
    .inner_join(users::table)
    .select((Project::as_select(), User::as_select()))
    .load::<(Project, User)>(conn)?;

// Left join (projects without tasks still included)
let results = projects::table
    .left_join(tasks::table)
    .select((Project::as_select(), Option::<Task>::as_select()))
    .load::<(Project, Option<Task>)>(conn)?;
```

**Filter Chaining**:
```rust
let results = tasks::table
    .filter(tasks::organization_id.eq(tenant_id))
    .filter(tasks::completed.eq(false))
    .filter(tasks::created_at.gt(last_week))
    .order_by(tasks::created_at.desc())
    .limit(50)
    .load(conn)?;
```

**Aggregations**:
```rust
use diesel::dsl::count;

let task_count: i64 = tasks::table
    .filter(tasks::project_id.eq(project_id))
    .count()
    .get_result(conn)?;
```

### Checkpoint Tests

```rust
#[test]
fn test_join_projects_with_users() {
    let mut conn = establish_connection();

    let org = create_organization(&mut conn, "TestOrg", "testorg").unwrap();
    let ctx = TenantContext::new(org.id);

    let user = ctx.create_user(&mut conn, "owner@example.com", "owner").unwrap();
    let project = ctx.create_project_with_owner(&mut conn, "Project", "Desc", user.id).unwrap();

    // Join projects with their owners
    let results = ctx.get_projects_with_owners(&mut conn).unwrap();

    assert_eq!(results.len(), 1);
    let (proj, owner) = &results[0];
    assert_eq!(proj.id, project.id);
    assert_eq!(owner.id, user.id);
}

#[test]
fn test_filter_chaining() {
    let mut conn = establish_connection();

    let org = create_organization(&mut conn, "TestOrg", "testorg").unwrap();
    let ctx = TenantContext::new(org.id);

    let project = ctx.create_project(&mut conn, "Project", "Desc").unwrap();

    // Create mix of completed and pending tasks
    let task1 = ctx.create_task(&mut conn, project.id, "Task 1").unwrap();
    let task2 = ctx.create_task(&mut conn, project.id, "Task 2").unwrap();
    ctx.complete_task(&mut conn, task1.id).unwrap();

    // Filter for pending tasks only
    let pending = ctx.get_pending_tasks(&mut conn).unwrap();

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, task2.id);
    assert!(!pending[0].completed);
}

#[test]
fn test_pagination() {
    let mut conn = establish_connection();

    let org = create_organization(&mut conn, "TestOrg", "testorg").unwrap();
    let ctx = TenantContext::new(org.id);

    let project = ctx.create_project(&mut conn, "Project", "Desc").unwrap();

    // Create 15 tasks
    for i in 0..15 {
        ctx.create_task(&mut conn, project.id, &format!("Task {}", i)).unwrap();
    }

    // Page 1 (10 tasks)
    let page1 = ctx.get_tasks_paginated(&mut conn, 10, 0).unwrap();
    assert_eq!(page1.len(), 10);

    // Page 2 (5 tasks)
    let page2 = ctx.get_tasks_paginated(&mut conn, 10, 10).unwrap();
    assert_eq!(page2.len(), 5);
}

#[test]
fn test_aggregation_count() {
    let mut conn = establish_connection();

    let org = create_organization(&mut conn, "TestOrg", "testorg").unwrap();
    let ctx = TenantContext::new(org.id);

    let project = ctx.create_project(&mut conn, "Project", "Desc").unwrap();

    ctx.create_task(&mut conn, project.id, "Task 1").unwrap();
    ctx.create_task(&mut conn, project.id, "Task 2").unwrap();
    ctx.create_task(&mut conn, project.id, "Task 3").unwrap();

    let count = ctx.count_project_tasks(&mut conn, project.id).unwrap();
    assert_eq!(count, 3);
}
```

### Starter Code

```rust
use diesel::dsl::count;

impl TenantContext {
    /// Get projects with owners (JOIN)
    fn get_projects_with_owners(&self, conn: &mut PgConnection) -> QueryResult<Vec<(Project, User)>> {
        // TODO: Inner join projects with users
        // Diesel syntax: projects::table.inner_join(users::table)
        todo!();
    }

    /// Get pending tasks (filter chaining)
    fn get_pending_tasks(&self, conn: &mut PgConnection) -> QueryResult<Vec<Task>> {
        // TODO: Filter by organization_id AND completed = false
        todo!(); // tasks::table.filter(tasks::organization_id.eq(self.organization_id)).filter(tasks::completed.eq(false)).load(conn)
    }

    /// Complete a task
    fn complete_task(&self, conn: &mut PgConnection, task_id: i32) -> QueryResult<Task> {
        // TODO: UPDATE task SET completed = true WHERE id = task_id AND organization_id = self.organization_id
        todo!(); // diesel::update(tasks::table.filter(tasks::id.eq(task_id)).filter(tasks::organization_id.eq(self.organization_id))).set(tasks::completed.eq(true)).returning(Task::as_returning()).get_result(conn)
    }

    /// Get tasks with pagination
    fn get_tasks_paginated(&self, conn: &mut PgConnection, limit: i64, offset: i64) -> QueryResult<Vec<Task>> {
        // TODO: Add limit and offset
        todo!(); // tasks::table.filter(tasks::organization_id.eq(self.organization_id)).order_by(tasks::created_at.desc()).limit(limit).offset(offset).load(conn)
    }

    /// Count tasks for a project
    fn count_project_tasks(&self, conn: &mut PgConnection, project_id: i32) -> QueryResult<i64> {
        // TODO: Use count() aggregation
        todo!(); // tasks::table.filter(tasks::project_id.eq(project_id)).filter(tasks::organization_id.eq(self.organization_id)).count().get_result(conn)
    }

    /// Get projects with task counts (GROUP BY)
    fn get_projects_with_task_counts(&self, conn: &mut PgConnection) -> QueryResult<Vec<(Project, i64)>> {
        // TODO: Left join with count aggregation
        // This is complex - Diesel's group_by syntax
        todo!();
    }
}
```

---

## Why Milestone 3 Isn't Enough → Moving to Milestone 4

**Missing Production Feature**: Schema evolution without downtime.

**The Problem**: Schema changes in production:
- Add column → deploy code → breaks old code expecting old schema
- Rename column → need to update all queries simultaneously
- No rollback if deployment fails

**What We're Adding**:
- **Diesel migrations**: Version-controlled up/down SQL
- **Migration testing**: Test both apply and rollback
- **Multi-phase deployments**: Add nullable column, backfill, make NOT NULL

**Improvement**:
- **Safety**: Test migrations before production
- **Rollback**: Revert failed deployments
- **History**: Git tracks all schema changes

---

## Milestone 4: Schema Migrations and Evolution

### Introduction

**The Problem**: Database schema changes over time:
- Add new features → new tables/columns
- Refactor → rename columns, change types
- Production → need rollback capability

**The Solution**: Diesel's migration system:
```bash
# Create migration
diesel migration generate add_task_priority

# Generates:
# migrations/2024-01-01-000000_add_task_priority/up.sql
# migrations/2024-01-01-000000_add_task_priority/down.sql
```

**up.sql** (apply change):
```sql
ALTER TABLE tasks ADD COLUMN priority INT NOT NULL DEFAULT 0;
CREATE INDEX idx_tasks_priority ON tasks(priority);
```

**down.sql** (rollback change):
```sql
DROP INDEX idx_tasks_priority;
ALTER TABLE tasks DROP COLUMN priority;
```

### Key Concepts

**Migration Workflow**:
1. `diesel migration generate <name>` - Create migration files
2. Edit `up.sql` and `down.sql`
3. `diesel migration run` - Apply migrations
4. `diesel migration revert` - Rollback last migration
5. Commit migration files to git

**Testing Migrations**:
```bash
# Test up and down
diesel migration run
diesel migration revert
diesel migration run
```

**Schema Versioning**:
- Each migration has timestamp (20240101000000)
- Applied in order by timestamp
- `__diesel_schema_migrations` table tracks applied migrations

### Checkpoint Tests

```rust
#[test]
fn test_migration_up_down() {
    // This is typically tested manually:
    // 1. diesel migration run
    // 2. Run tests (should pass)
    // 3. diesel migration revert
    // 4. Run old tests (should still pass)

    // Automated migration testing requires test database setup
    let mut conn = establish_connection();

    // After migration: priority column should exist
    let task = create_task_with_priority(&mut conn, 1, 1, "High priority", 10).unwrap();
    assert_eq!(task.priority, 10);
}

#[test]
fn test_multi_phase_migration() {
    // Phase 1: Add nullable column
    // Phase 2: Backfill data
    // Phase 3: Make NOT NULL
    // This tests safe production migrations
}
```

### Starter Code

**Migration 1: Add priority column**
```sql
-- migrations/20240101000000_add_task_priority/up.sql
ALTER TABLE tasks ADD COLUMN priority INT NOT NULL DEFAULT 0;
CREATE INDEX idx_tasks_priority ON tasks(priority);

-- migrations/20240101000000_add_task_priority/down.sql
DROP INDEX idx_tasks_priority;
ALTER TABLE tasks DROP COLUMN priority;
```

**Migration 2: Add due dates**
```sql
-- migrations/20240101000001_add_due_dates/up.sql
ALTER TABLE tasks ADD COLUMN due_at TIMESTAMPTZ;
CREATE INDEX idx_tasks_due_at ON tasks(due_at) WHERE due_at IS NOT NULL;

-- migrations/20240101000001_add_due_dates/down.sql
DROP INDEX idx_tasks_due_at;
ALTER TABLE tasks DROP COLUMN due_at;
```

**Migration 3: Multi-phase (safe production)**
```sql
-- Phase 1 (deploy with code that handles both old and new schema)
-- migrations/20240101000002_add_status_phase1/up.sql
ALTER TABLE tasks ADD COLUMN status VARCHAR(20);

-- Phase 2 (backfill existing data)
-- migrations/20240101000003_add_status_phase2/up.sql
UPDATE tasks SET status = CASE
    WHEN completed THEN 'completed'
    ELSE 'pending'
END
WHERE status IS NULL;

-- Phase 3 (enforce constraint)
-- migrations/20240101000004_add_status_phase3/up.sql
ALTER TABLE tasks ALTER COLUMN status SET NOT NULL;
ALTER TABLE tasks ALTER COLUMN status SET DEFAULT 'pending';
```

**Updated Schema**:
```rust
// After running migrations, regenerate schema:
// diesel print-schema > src/schema.rs

// schema.rs now has new columns
diesel::table! {
    tasks (id) {
        id -> Int4,
        organization_id -> Int4,
        project_id -> Int4,
        title -> Varchar,
        completed -> Bool,
        priority -> Int4,  // NEW
        due_at -> Nullable<Timestamptz>,  // NEW
        status -> Varchar,  // NEW
        created_at -> Timestamptz,
    }
}
```

**Updated Structs**:
```rust
#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = tasks)]
struct Task {
    id: i32,
    organization_id: i32,
    project_id: i32,
    title: String,
    completed: bool,
    priority: i32,
    due_at: Option<chrono::NaiveDateTime>,
    status: String,
    created_at: chrono::NaiveDateTime,
}
```

---

## Why Milestone 4 Isn't Enough → Moving to Milestone 5

**Missing Concurrency Handling**: Multiple users editing same task simultaneously:
- User A reads task (version 1)
- User B reads task (version 1)
- User A updates task (version 2)
- User B updates task (overwrites A's changes!) ← Lost update

**What We're Adding**:
- **Optimistic locking**: `version` column incremented on each update
- **Conflict detection**: Update only if version matches
- **Retry logic**: Client retries on version mismatch

**Improvement**:
- **Correctness**: No lost updates
- **Performance**: No pessimistic locks (better concurrency)
- **Standard pattern**: Used in Salesforce, Stripe, many SaaS

---

## Milestone 5: Optimistic Locking for Concurrent Edits

### Introduction

**The Problem**: Concurrent edits cause lost updates:
```
Time  User A                User B
1     Read task (v1)        -
2     -                     Read task (v1)
3     Update task (v2)      -
4     -                     Update task (overwrites A's change!)
```

**The Solution**: Add `version` column:
```
Time  User A                           User B
1     Read task (v1)                   -
2     -                                Read task (v1)
3     UPDATE WHERE id=1 AND version=1  -
      SET ..., version=2
4     -                                UPDATE WHERE id=1 AND version=1
                                       → 0 rows affected (conflict!)
                                       → Refresh and retry
```

### Key Concepts

**Schema Change**:
```sql
ALTER TABLE tasks ADD COLUMN version INT NOT NULL DEFAULT 1;
```

**Update Pattern**:
```rust
// Update with version check
let rows_affected = diesel::update(tasks::table)
    .filter(tasks::id.eq(task_id))
    .filter(tasks::version.eq(current_version))  // Key: Check version!
    .set((
        tasks::title.eq(new_title),
        tasks::version.eq(current_version + 1),  // Increment version
    ))
    .execute(conn)?;

if rows_affected == 0 {
    return Err("Concurrent modification detected".into());
}
```

**Client Retry Logic**:
```rust
loop {
    let task = get_task(conn, task_id)?;

    match update_task_optimistic(conn, task.id, task.version, new_data) {
        Ok(_) => break,  // Success
        Err(_) => {
            println!("Conflict detected, retrying...");
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}
```

### Checkpoint Tests

```rust
#[test]
fn test_optimistic_locking_success() {
    let mut conn = establish_connection();

    let org = create_organization(&mut conn, "TestOrg", "testorg").unwrap();
    let ctx = TenantContext::new(org.id);

    let project = ctx.create_project(&mut conn, "Project", "Desc").unwrap();
    let task = ctx.create_task(&mut conn, project.id, "Task").unwrap();

    assert_eq!(task.version, 1);

    // Update with correct version
    let updated = ctx.update_task_title(&mut conn, task.id, task.version, "Updated").unwrap();
    assert_eq!(updated.version, 2);
    assert_eq!(updated.title, "Updated");
}

#[test]
fn test_optimistic_locking_conflict() {
    let mut conn = establish_connection();

    let org = create_organization(&mut conn, "TestOrg", "testorg").unwrap();
    let ctx = TenantContext::new(org.id);

    let project = ctx.create_project(&mut conn, "Project", "Desc").unwrap();
    let task = ctx.create_task(&mut conn, project.id, "Task").unwrap();

    // Simulate concurrent update
    ctx.update_task_title(&mut conn, task.id, task.version, "Update A").unwrap();

    // Try to update with stale version
    let result = ctx.update_task_title(&mut conn, task.id, task.version, "Update B");

    assert!(result.is_err()); // Should fail (version mismatch)
}

#[test]
fn test_retry_on_conflict() {
    let mut conn = establish_connection();

    let org = create_organization(&mut conn, "TestOrg", "testorg").unwrap();
    let ctx = TenantContext::new(org.id);

    let project = ctx.create_project(&mut conn, "Project", "Desc").unwrap();
    let task = ctx.create_task(&mut conn, project.id, "Task").unwrap();

    // Simulate two clients updating
    std::thread::spawn(move || {
        let mut conn2 = establish_connection();
        let ctx2 = TenantContext::new(org.id);
        ctx2.update_task_with_retry(&mut conn2, task.id, "Client 1").unwrap();
    });

    let result = ctx.update_task_with_retry(&mut conn, task.id, "Client 2");
    assert!(result.is_ok()); // Retry should succeed
}
```

### Starter Code

```rust
impl TenantContext {
    /// Update task title with optimistic locking
    fn update_task_title(
        &self,
        conn: &mut PgConnection,
        task_id: i32,
        expected_version: i32,
        new_title: &str,
    ) -> QueryResult<Task> {
        // TODO: UPDATE with version check
        let rows_affected = diesel::update(tasks::table)
            .filter(tasks::id.eq(task_id))
            .filter(tasks::organization_id.eq(self.organization_id))
            .filter(tasks::version.eq(expected_version))  // Optimistic lock
            .set((
                tasks::title.eq(new_title),
                tasks::version.eq(expected_version + 1),  // Increment
            ))
            .execute(conn)?;

        if rows_affected == 0 {
            return Err(diesel::result::Error::NotFound);
        }

        // Fetch updated task
        tasks::table
            .filter(tasks::id.eq(task_id))
            .first(conn)
    }

    /// Update with automatic retry on conflict
    fn update_task_with_retry(
        &self,
        conn: &mut PgConnection,
        task_id: i32,
        new_title: &str,
    ) -> QueryResult<Task> {
        const MAX_RETRIES: usize = 5;

        for attempt in 0..MAX_RETRIES {
            // Fetch current task
            let task = tasks::table
                .filter(tasks::id.eq(task_id))
                .filter(tasks::organization_id.eq(self.organization_id))
                .first::<Task>(conn)?;

            // Try to update
            match self.update_task_title(conn, task.id, task.version, new_title) {
                Ok(updated) => return Ok(updated),
                Err(diesel::result::Error::NotFound) => {
                    // Conflict detected, retry
                    if attempt < MAX_RETRIES - 1 {
                        std::thread::sleep(std::time::Duration::from_millis(50 * (attempt as u64 + 1)));
                        continue;
                    } else {
                        return Err(diesel::result::Error::RollbackTransaction);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Err(diesel::result::Error::RollbackTransaction)
    }
}
```

---

## Why Milestone 5 Isn't Enough → Moving to Milestone 6

**Missing Operational Features**:
1. **No connection pooling**: Creating connection per request is slow
2. **No database seeding**: Fresh database is empty, can't test
3. **No backup/restore**: Accidents happen, need recovery

**What We're Adding**:
- **Connection pooling with r2d2**: Reuse connections efficiently
- **Database seeding**: Populate test data
- **Backup scripts**: Production safety net

---

## Milestone 6: Production Patterns (Pooling, Seeding, Backups)

### Introduction

**Production Requirements**:
- **Connection pooling**: Expensive to create connections (50-200ms each)
- **Seeding**: Developers need consistent test data
- **Backups**: Production databases need disaster recovery

### Key Concepts

**Connection Pooling with r2d2**:
```rust
use diesel::r2d2::{self, ConnectionManager};

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

fn create_pool(database_url: &str) -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .max_size(15)
        .build(manager)
        .expect("Failed to create pool")
}

// Usage
let pool = create_pool(&database_url);
let mut conn = pool.get().unwrap();  // Borrow from pool
// Connection automatically returned on drop
```

**Database Seeding**:
```rust
fn seed_database(conn: &mut PgConnection) -> QueryResult<()> {
    // Create test organizations
    let org1 = diesel::insert_into(organizations::table)
        .values(&NewOrganization { name: "Demo Org", slug: "demo" })
        .returning(Organization::as_returning())
        .get_result(conn)?;

    // Create test users
    diesel::insert_into(users::table)
        .values(&NewUser {
            organization_id: org1.id,
            email: "admin@demo.com",
            username: "admin",
        })
        .execute(conn)?;

    Ok(())
}
```

**Backup Scripts**:
```bash
#!/bin/bash
# scripts/backup.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="backups/backup_${TIMESTAMP}.sql"

pg_dump $DATABASE_URL > $BACKUP_FILE
gzip $BACKUP_FILE

echo "Backup created: ${BACKUP_FILE}.gz"
```

### Checkpoint Tests

```rust
#[test]
fn test_connection_pool() {
    let pool = create_pool(&database_url());

    // Acquire multiple connections
    let conn1 = pool.get().unwrap();
    let conn2 = pool.get().unwrap();

    // Both work independently
    assert!(conn1.is_valid());
    assert!(conn2.is_valid());

    drop(conn1);
    drop(conn2);

    // Connections returned to pool
    assert_eq!(pool.state().connections, 2);
}

#[test]
fn test_database_seeding() {
    let mut conn = establish_connection();

    // Seed database
    seed_database(&mut conn).unwrap();

    // Verify seeded data exists
    let org = organizations::table
        .filter(organizations::slug.eq("demo"))
        .first::<Organization>(&mut conn)
        .unwrap();

    assert_eq!(org.name, "Demo Org");
}
```

### Starter Code

```rust
use diesel::r2d2::{self, ConnectionManager, Pool};

type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Create connection pool
fn create_pool(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .max_size(15)
        .min_idle(Some(5))
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)
        .expect("Failed to create pool")
}

/// Seed database with test data
fn seed_database(conn: &mut PgConnection) -> QueryResult<()> {
    use crate::schema::{organizations, users, projects};

    conn.transaction(|conn| {
        // TODO: Create organizations
        let org1 = diesel::insert_into(organizations::table)
            .values(&NewOrganization { name: "Demo Org", slug: "demo" })
            .returning(Organization::as_returning())
            .get_result(conn)?;

        let org2 = diesel::insert_into(organizations::table)
            .values(&NewOrganization { name: "Test Corp", slug: "test" })
            .returning(Organization::as_returning())
            .get_result(conn)?;

        // TODO: Create users
        diesel::insert_into(users::table)
            .values(vec![
                NewUser { organization_id: org1.id, email: "admin@demo.com", username: "admin" },
                NewUser { organization_id: org1.id, email: "user@demo.com", username: "user" },
                NewUser { organization_id: org2.id, email: "admin@test.com", username: "testadmin" },
            ])
            .execute(conn)?;

        // TODO: Create projects
        diesel::insert_into(projects::table)
            .values(vec![
                NewProject { organization_id: org1.id, name: "MVP Project", description: "Initial launch" },
                NewProject { organization_id: org1.id, name: "Feature X", description: "New feature" },
            ])
            .execute(conn)?;

        Ok(())
    })
}

/// CLI for database operations
fn main() {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = create_pool(&database_url);

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("seed") => {
            let mut conn = pool.get().unwrap();
            seed_database(&mut conn).expect("Failed to seed database");
            println!("Database seeded successfully");
        }
        Some("reset") => {
            // Drop and recreate database (use with caution!)
            println!("This would reset the database");
        }
        _ => {
            println!("Usage: {} [seed|reset]", args[0]);
        }
    }
}
```

**Backup Script**:
```bash
#!/bin/bash
# scripts/backup.sh

set -e

DATABASE_URL=${DATABASE_URL:?"DATABASE_URL must be set"}
BACKUP_DIR="backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/backup_${TIMESTAMP}.sql"

mkdir -p $BACKUP_DIR

echo "Backing up database..."
pg_dump $DATABASE_URL > $BACKUP_FILE

echo "Compressing backup..."
gzip $BACKUP_FILE

echo "Backup created: ${BACKUP_FILE}.gz"
echo "Size: $(du -h ${BACKUP_FILE}.gz | cut -f1)"

# Keep only last 7 backups
ls -t ${BACKUP_DIR}/backup_*.sql.gz | tail -n +8 | xargs -r rm

echo "Cleanup complete. Kept 7 most recent backups."
```

**Restore Script**:
```bash
#!/bin/bash
# scripts/restore.sh

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <backup_file.sql.gz>"
    exit 1
fi

BACKUP_FILE=$1
DATABASE_URL=${DATABASE_URL:?"DATABASE_URL must be set"}

echo "Restoring from $BACKUP_FILE..."

gunzip -c $BACKUP_FILE | psql $DATABASE_URL

echo "Restore complete"
```

---

### Testing Strategies

1. **Tenant Isolation Tests**:
   - Create data in multiple tenants
   - Verify each tenant sees only their data
   - Attempt cross-tenant access (should fail)

2. **Migration Testing**:
   - Run all migrations up
   - Run tests
   - Revert all migrations down
   - Re-apply migrations up
   - Verify idempotency

3. **Optimistic Locking Tests**:
   - Simulate concurrent updates
   - Verify conflicts detected
   - Test retry logic succeeds

4. **Connection Pool Tests**:
   - Stress test with many concurrent queries
   - Verify pool doesn't exhaust
   - Test connection timeout handling

5. **Seed Data Verification**:
   - Seed fresh database
   - Verify all expected data exists
   - Test application against seeded data

---

### Complete Working Example

This multi-tenant SaaS database demonstrates:
- **Diesel's type-safe DSL** - No SQL strings, pure Rust queries
- **Multi-tenant architecture** - Tenant isolation with organization_id filtering
- **Complex queries** - JOINs, filters, aggregations, pagination
- **Schema migrations** - Version-controlled database evolution
- **Optimistic locking** - Concurrent edit conflict detection
- **Production patterns** - Connection pooling, seeding, backups

**Production Deployment**:
```bash
# Setup
diesel setup
diesel migration run

# Seed database
cargo run -- seed

# Backup (cron job)
./scripts/backup.sh

# Monitor pool health
# (Log pool.state() periodically)
```

**API Integration** (with axum):
```rust
// Extract tenant from JWT/session
struct TenantExtractor(TenantContext);

#[axum::async_trait]
impl FromRequestParts for TenantExtractor {
    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self> {
        // Extract tenant_id from auth token
        let tenant_id = extract_tenant_from_auth(parts)?;
        Ok(TenantExtractor(TenantContext::new(tenant_id)))
    }
}

// Use in handlers
async fn get_projects(
    TenantExtractor(ctx): TenantExtractor,
    State(pool): State<DbPool>,
) -> Result<Json<Vec<Project>>> {
    let mut conn = pool.get()?;
    let projects = ctx.get_projects(&mut conn)?;
    Ok(Json(projects))
}
```

This implementation is production-ready for multi-tenant SaaS applications with:
- **Security**: Tenant isolation enforced by type system
- **Type safety**: Diesel catches errors at compile time
- **Performance**: Connection pooling, indexed queries
- **Scalability**: Shared database serves thousands of tenants
- **Reliability**: Migrations, backups, optimistic locking
