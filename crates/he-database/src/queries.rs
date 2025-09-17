//! Database queries

use crate::models::*;
use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserQueries;

impl UserQueries {
    pub async fn create_user(
        pool: &PgPool,
        login: &str,
        email: &str,
        password: &str,
    ) -> Result<User> {
        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?
            .to_string();

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (login, pwd, email, online, last_login, created, last_act, last_ip)
            VALUES ($1, $2, $3, false, NOW(), NOW(), NOW(), '127.0.0.1')
            RETURNING *
            "#,
            login,
            password_hash,
            email
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_id(pool: &PgPool, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    pub async fn verify_password(user: &User, password: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(&user.pwd)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub async fn update_last_login(pool: &PgPool, user_id: i64, ip: &str) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET last_login = NOW(), last_ip = $1, online = true WHERE id = $2",
            ip,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_offline(pool: &PgPool, user_id: i64) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET online = false WHERE id = $1",
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

pub struct ProcessQueries;

impl ProcessQueries {
    pub async fn create_process(
        pool: &PgPool,
        user_id: i64,
        process_type: &str,
        pc_id: &str,
        target_pc_id: Option<String>,
    ) -> Result<Process> {
        let process = sqlx::query_as!(
            Process,
            r#"
            INSERT INTO processes (user_id, pc_id, target_pc_id, process_type, priority, start_time, end_time)
            VALUES ($1, $2, $3, $4, 0, NOW(), NOW() + INTERVAL '1 minute')
            RETURNING *
            "#,
            user_id,
            pc_id,
            target_pc_id,
            process_type
        )
        .fetch_one(pool)
        .await?;

        Ok(process)
    }

    pub async fn create_process_with_duration(
        pool: &PgPool,
        user_id: i64,
        process_type: &str,
        pc_id: &str,
        target_pc_id: Option<String>,
        duration_seconds: i32,
    ) -> Result<Process> {
        let process = sqlx::query_as!(
            Process,
            r#"
            INSERT INTO processes (user_id, pc_id, target_pc_id, process_type, priority, start_time, end_time)
            VALUES ($1, $2, $3, $4, 0, NOW(), NOW() + make_interval(secs => $5))
            RETURNING *
            "#,
            user_id,
            pc_id,
            target_pc_id,
            process_type,
            duration_seconds as f64
        )
        .fetch_one(pool)
        .await?;

        Ok(process)
    }

    pub async fn get_user_processes(pool: &PgPool, user_id: i64) -> Result<Vec<Process>> {
        let processes = sqlx::query_as!(
            Process,
            "SELECT * FROM processes WHERE user_id = $1 AND end_time > NOW() ORDER BY priority DESC",
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(processes)
    }

    pub async fn cancel_process(pool: &PgPool, pid: i64, user_id: i64) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM processes WHERE pid = $1 AND user_id = $2",
            pid,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

pub struct HardwareQueries;

impl HardwareQueries {
    pub async fn get_hardware(pool: &PgPool, pc_id: &str) -> Result<Option<Hardware>> {
        let hardware = sqlx::query_as!(
            Hardware,
            "SELECT * FROM hardware WHERE pc_id = $1",
            pc_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(hardware)
    }

    pub async fn get_user_hardware(pool: &PgPool, user_id: i64) -> Result<Hardware> {
        // Try to get existing hardware or create default one
        let hardware = sqlx::query_as!(
            Hardware,
            "SELECT * FROM hardware WHERE user_id = $1 LIMIT 1",
            user_id
        )
        .fetch_optional(pool)
        .await?;

        match hardware {
            Some(hw) => Ok(hw),
            None => {
                // Create default hardware for user
                let hw = sqlx::query_as!(
                    Hardware,
                    r#"
                    INSERT INTO hardware (user_id, cpu_mhz, ram_mb, hdd_mb, net_mbps, gpu_cores, total_slots, used_slots)
                    VALUES ($1, 1000, 1024, 10240, 10, 1, 5, 1)
                    RETURNING *
                    "#,
                    user_id
                )
                .fetch_one(pool)
                .await?;
                Ok(hw)
            }
        }
    }

    pub async fn update_hardware(
        pool: &PgPool,
        pc_id: &str,
        cpu_speed: Option<f64>,
        ram_size: Option<i64>,
        hdd_size: Option<i64>,
        net_speed: Option<f64>,
    ) -> Result<()> {
        let mut query = String::from("UPDATE hardware SET ");
        let mut updates = Vec::new();

        if let Some(cpu) = cpu_speed {
            updates.push(format!("cpu_speed = {}", cpu));
        }
        if let Some(ram) = ram_size {
            updates.push(format!("ram_size = {}", ram));
        }
        if let Some(hdd) = hdd_size {
            updates.push(format!("hdd_size = {}", hdd));
        }
        if let Some(net) = net_speed {
            updates.push(format!("net_speed = {}", net));
        }

        if updates.is_empty() {
            return Ok(());
        }

        query.push_str(&updates.join(", "));
        query.push_str(&format!(" WHERE pc_id = '{}'", pc_id));

        sqlx::query(&query)
            .execute(pool)
            .await?;

        Ok(())
    }
}

pub struct BankQueries;

impl BankQueries {
    pub async fn get_user_accounts(pool: &PgPool, user_id: i64) -> Result<Vec<BankAccount>> {
        let accounts = sqlx::query_as!(
            BankAccount,
            "SELECT * FROM bank_accounts WHERE user_id = $1",
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(accounts)
    }

    pub async fn transfer_money(
        pool: &PgPool,
        from_account: &str,
        to_account: &str,
        amount: i64,
    ) -> Result<bool> {
        let mut tx = pool.begin().await?;

        // Deduct from sender
        let result = sqlx::query!(
            "UPDATE bank_accounts SET balance = balance - $1 WHERE account_number = $2 AND balance >= $1",
            amount,
            from_account
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Ok(false);
        }

        // Add to receiver
        sqlx::query!(
            "UPDATE bank_accounts SET balance = balance + $1 WHERE account_number = $2",
            amount,
            to_account
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(true)
    }
}

pub struct MissionQueries;

impl MissionQueries {
    pub async fn get_user_missions(pool: &PgPool, user_id: i64) -> Result<Vec<Mission>> {
        let missions = sqlx::query_as!(
            Mission,
            "SELECT * FROM missions WHERE user_id = $1 ORDER BY id DESC",
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(missions)
    }

    pub async fn update_mission_progress(
        pool: &PgPool,
        mission_id: i64,
        progress: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE missions
            SET progress = $1,
                status = CASE
                    WHEN $1 >= total_steps THEN 'completed'
                    ELSE status
                END,
                completed_at = CASE
                    WHEN $1 >= total_steps THEN NOW()
                    ELSE completed_at
                END
            WHERE id = $2
            "#,
            progress,
            mission_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}