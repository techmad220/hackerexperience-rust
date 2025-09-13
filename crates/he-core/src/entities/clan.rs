use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use chrono::{DateTime, Utc};

use crate::error::Result;

/// Represents a clan in the game
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Clan {
    pub id: i32,
    pub name: String,
    pub nick: String,
    pub desc: Option<String>,
    pub leader_id: i32,
    pub member_count: i32,
    pub created_at: DateTime<Utc>,
    pub logo: Option<String>,
    pub founded_by: i32,
    pub website: Option<String>,
    pub recruiting: bool,
    pub req_level: i32,
    pub req_reputation: i32,
}

/// Clan member information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClanMember {
    pub user_id: i32,
    pub clan_id: i32,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub contribution: i32,
    pub login: String,
}

/// Clan invitation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClanInvitation {
    pub id: i32,
    pub clan_id: i32,
    pub user_id: i32,
    pub invited_by: i32,
    pub invited_at: DateTime<Utc>,
    pub message: Option<String>,
}

/// Clan war information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClanWar {
    pub id: i32,
    pub clan_a: i32,
    pub clan_b: i32,
    pub started_by: i32,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub winner: Option<i32>,
    pub clan_a_score: i32,
    pub clan_b_score: i32,
}

#[async_trait]
pub trait ClanRepository {
    /// Check if a player belongs to a clan
    async fn player_has_clan(&self, user_id: i32) -> Result<bool>;
    
    /// Get the clan ID of a player
    async fn get_player_clan(&self, user_id: i32) -> Result<Option<i32>>;
    
    /// Get clan information by ID
    async fn get_clan_info(&self, clan_id: i32) -> Result<Option<Clan>>;
    
    /// Check if clan exists
    async fn clan_exists(&self, clan_id: i32) -> Result<bool>;
    
    /// Create a new clan
    async fn create_clan(&self, name: &str, nick: &str, desc: Option<&str>, 
                        founder_id: i32) -> Result<i32>;
    
    /// Get clan members
    async fn get_clan_members(&self, clan_id: i32) -> Result<Vec<ClanMember>>;
    
    /// Add member to clan
    async fn add_member(&self, user_id: i32, clan_id: i32) -> Result<()>;
    
    /// Remove member from clan
    async fn remove_member(&self, user_id: i32, clan_id: i32) -> Result<()>;
    
    /// Check if user is clan leader
    async fn is_clan_leader(&self, user_id: i32, clan_id: i32) -> Result<bool>;
    
    /// Update clan information
    async fn update_clan(&self, clan_id: i32, name: Option<&str>, 
                        nick: Option<&str>, desc: Option<&str>, 
                        website: Option<&str>) -> Result<()>;
    
    /// Send clan invitation
    async fn send_invitation(&self, clan_id: i32, user_id: i32, 
                           invited_by: i32, message: Option<&str>) -> Result<()>;
    
    /// Get pending invitations for user
    async fn get_user_invitations(&self, user_id: i32) -> Result<Vec<ClanInvitation>>;
    
    /// Accept clan invitation
    async fn accept_invitation(&self, invitation_id: i32) -> Result<()>;
    
    /// Reject clan invitation
    async fn reject_invitation(&self, invitation_id: i32) -> Result<()>;
    
    /// Start clan war
    async fn start_war(&self, clan_a: i32, clan_b: i32, started_by: i32) -> Result<i32>;
    
    /// Get active clan wars
    async fn get_active_wars(&self, clan_id: i32) -> Result<Vec<ClanWar>>;
    
    /// Update war score
    async fn update_war_score(&self, war_id: i32, clan_id: i32, points: i32) -> Result<()>;
    
    /// End clan war
    async fn end_war(&self, war_id: i32, winner: Option<i32>) -> Result<()>;
    
    /// Get clan rankings
    async fn get_clan_rankings(&self, limit: i32) -> Result<Vec<Clan>>;
    
    /// Update member role
    async fn update_member_role(&self, user_id: i32, clan_id: i32, role: &str) -> Result<()>;
    
    /// Check if clan name is available
    async fn is_clan_name_available(&self, name: &str) -> Result<bool>;
    
    /// Check if clan nick is available  
    async fn is_clan_nick_available(&self, nick: &str) -> Result<bool>;
    
    /// Disband clan
    async fn disband_clan(&self, clan_id: i32) -> Result<()>;
}

pub struct ClanService {
    db: PgPool,
}

impl ClanService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ClanRepository for ClanService {
    /// Check if a player belongs to a clan
    async fn player_has_clan(&self, user_id: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM clan_members WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Get the clan ID of a player
    async fn get_player_clan(&self, user_id: i32) -> Result<Option<i32>> {
        let clan_id = sqlx::query_scalar!(
            "SELECT clan_id FROM clan_members WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(clan_id)
    }
    
    /// Get clan information by ID
    async fn get_clan_info(&self, clan_id: i32) -> Result<Option<Clan>> {
        let clan = sqlx::query_as!(
            Clan,
            "SELECT c.id, c.name, c.nick, c.desc, c.leader_id, 
                    COUNT(cm.user_id) as member_count, c.created_at, c.logo,
                    c.founded_by, c.website, c.recruiting, c.req_level, c.req_reputation
             FROM clans c
             LEFT JOIN clan_members cm ON c.id = cm.clan_id
             WHERE c.id = $1
             GROUP BY c.id",
            clan_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(clan)
    }
    
    /// Check if clan exists
    async fn clan_exists(&self, clan_id: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM clans WHERE id = $1",
            clan_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Create a new clan
    async fn create_clan(&self, name: &str, nick: &str, desc: Option<&str>, 
                        founder_id: i32) -> Result<i32> {
        let clan_id = sqlx::query_scalar!(
            "INSERT INTO clans (name, nick, desc, leader_id, founded_by, created_at, recruiting, req_level, req_reputation)
             VALUES ($1, $2, $3, $4, $4, NOW(), true, 1, 0)
             RETURNING id",
            name,
            nick,
            desc,
            founder_id
        )
        .fetch_one(&self.db)
        .await?;
        
        // Add founder as first member with leader role
        self.add_member(founder_id, clan_id).await?;
        self.update_member_role(founder_id, clan_id, "leader").await?;
        
        Ok(clan_id)
    }
    
    /// Get clan members
    async fn get_clan_members(&self, clan_id: i32) -> Result<Vec<ClanMember>> {
        let members = sqlx::query_as!(
            ClanMember,
            "SELECT cm.user_id, cm.clan_id, cm.role, cm.joined_at, cm.contribution, u.login
             FROM clan_members cm
             INNER JOIN users u ON cm.user_id = u.id
             WHERE cm.clan_id = $1
             ORDER BY cm.role, cm.joined_at",
            clan_id
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(members)
    }
    
    /// Add member to clan
    async fn add_member(&self, user_id: i32, clan_id: i32) -> Result<()> {
        sqlx::query!(
            "INSERT INTO clan_members (user_id, clan_id, role, joined_at, contribution)
             VALUES ($1, $2, 'member', NOW(), 0)",
            user_id,
            clan_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Remove member from clan
    async fn remove_member(&self, user_id: i32, clan_id: i32) -> Result<()> {
        sqlx::query!(
            "DELETE FROM clan_members WHERE user_id = $1 AND clan_id = $2",
            user_id,
            clan_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Check if user is clan leader
    async fn is_clan_leader(&self, user_id: i32, clan_id: i32) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM clan_members 
             WHERE user_id = $1 AND clan_id = $2 AND role = 'leader'",
            user_id,
            clan_id
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) > 0)
    }
    
    /// Update clan information
    async fn update_clan(&self, clan_id: i32, name: Option<&str>, 
                        nick: Option<&str>, desc: Option<&str>, 
                        website: Option<&str>) -> Result<()> {
        let mut query = String::from("UPDATE clans SET ");
        let mut params: Vec<String> = Vec::new();
        let mut param_count = 1;
        
        if let Some(name) = name {
            params.push(format!("name = ${}", param_count));
            param_count += 1;
        }
        if let Some(nick) = nick {
            params.push(format!("nick = ${}", param_count));
            param_count += 1;
        }
        if let Some(desc) = desc {
            params.push(format!("desc = ${}", param_count));
            param_count += 1;
        }
        if let Some(website) = website {
            params.push(format!("website = ${}", param_count));
            param_count += 1;
        }
        
        if params.is_empty() {
            return Ok(());
        }
        
        query.push_str(&params.join(", "));
        query.push_str(&format!(" WHERE id = ${}", param_count));
        
        let mut q = sqlx::query(&query);
        
        if let Some(name) = name {
            q = q.bind(name);
        }
        if let Some(nick) = nick {
            q = q.bind(nick);
        }
        if let Some(desc) = desc {
            q = q.bind(desc);
        }
        if let Some(website) = website {
            q = q.bind(website);
        }
        q = q.bind(clan_id);
        
        q.execute(&self.db).await?;
        
        Ok(())
    }
    
    /// Send clan invitation
    async fn send_invitation(&self, clan_id: i32, user_id: i32, 
                           invited_by: i32, message: Option<&str>) -> Result<()> {
        sqlx::query!(
            "INSERT INTO clan_invitations (clan_id, user_id, invited_by, invited_at, message)
             VALUES ($1, $2, $3, NOW(), $4)",
            clan_id,
            user_id,
            invited_by,
            message
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Get pending invitations for user
    async fn get_user_invitations(&self, user_id: i32) -> Result<Vec<ClanInvitation>> {
        let invitations = sqlx::query_as!(
            ClanInvitation,
            "SELECT id, clan_id, user_id, invited_by, invited_at, message
             FROM clan_invitations
             WHERE user_id = $1
             ORDER BY invited_at DESC",
            user_id
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(invitations)
    }
    
    /// Accept clan invitation
    async fn accept_invitation(&self, invitation_id: i32) -> Result<()> {
        let invitation = sqlx::query!(
            "SELECT clan_id, user_id FROM clan_invitations WHERE id = $1",
            invitation_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        if let Some(inv) = invitation {
            self.add_member(inv.user_id, inv.clan_id).await?;
            
            sqlx::query!(
                "DELETE FROM clan_invitations WHERE id = $1",
                invitation_id
            )
            .execute(&self.db)
            .await?;
        }
        
        Ok(())
    }
    
    /// Reject clan invitation
    async fn reject_invitation(&self, invitation_id: i32) -> Result<()> {
        sqlx::query!(
            "DELETE FROM clan_invitations WHERE id = $1",
            invitation_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Start clan war
    async fn start_war(&self, clan_a: i32, clan_b: i32, started_by: i32) -> Result<i32> {
        let war_id = sqlx::query_scalar!(
            "INSERT INTO clan_wars (clan_a, clan_b, started_by, started_at, clan_a_score, clan_b_score)
             VALUES ($1, $2, $3, NOW(), 0, 0)
             RETURNING id",
            clan_a,
            clan_b,
            started_by
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(war_id)
    }
    
    /// Get active clan wars
    async fn get_active_wars(&self, clan_id: i32) -> Result<Vec<ClanWar>> {
        let wars = sqlx::query_as!(
            ClanWar,
            "SELECT id, clan_a, clan_b, started_by, started_at, ended_at, winner, clan_a_score, clan_b_score
             FROM clan_wars
             WHERE (clan_a = $1 OR clan_b = $1) AND ended_at IS NULL
             ORDER BY started_at DESC",
            clan_id
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(wars)
    }
    
    /// Update war score
    async fn update_war_score(&self, war_id: i32, clan_id: i32, points: i32) -> Result<()> {
        sqlx::query!(
            "UPDATE clan_wars 
             SET clan_a_score = CASE WHEN clan_a = $2 THEN clan_a_score + $3 ELSE clan_a_score END,
                 clan_b_score = CASE WHEN clan_b = $2 THEN clan_b_score + $3 ELSE clan_b_score END
             WHERE id = $1",
            war_id,
            clan_id,
            points
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// End clan war
    async fn end_war(&self, war_id: i32, winner: Option<i32>) -> Result<()> {
        sqlx::query!(
            "UPDATE clan_wars SET ended_at = NOW(), winner = $2 WHERE id = $1",
            war_id,
            winner
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Get clan rankings
    async fn get_clan_rankings(&self, limit: i32) -> Result<Vec<Clan>> {
        let clans = sqlx::query_as!(
            Clan,
            "SELECT c.id, c.name, c.nick, c.desc, c.leader_id, 
                    COUNT(cm.user_id) as member_count, c.created_at, c.logo,
                    c.founded_by, c.website, c.recruiting, c.req_level, c.req_reputation
             FROM clans c
             LEFT JOIN clan_members cm ON c.id = cm.clan_id
             GROUP BY c.id
             ORDER BY member_count DESC, c.created_at ASC
             LIMIT $1",
            limit
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(clans)
    }
    
    /// Update member role
    async fn update_member_role(&self, user_id: i32, clan_id: i32, role: &str) -> Result<()> {
        sqlx::query!(
            "UPDATE clan_members SET role = $3 WHERE user_id = $1 AND clan_id = $2",
            user_id,
            clan_id,
            role
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Check if clan name is available
    async fn is_clan_name_available(&self, name: &str) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM clans WHERE LOWER(name) = LOWER($1)",
            name
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) == 0)
    }
    
    /// Check if clan nick is available  
    async fn is_clan_nick_available(&self, nick: &str) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM clans WHERE LOWER(nick) = LOWER($1)",
            nick
        )
        .fetch_one(&self.db)
        .await?;
        
        Ok(count.unwrap_or(0) == 0)
    }
    
    /// Disband clan
    async fn disband_clan(&self, clan_id: i32) -> Result<()> {
        // Remove all members
        sqlx::query!(
            "DELETE FROM clan_members WHERE clan_id = $1",
            clan_id
        )
        .execute(&self.db)
        .await?;
        
        // Remove all invitations
        sqlx::query!(
            "DELETE FROM clan_invitations WHERE clan_id = $1",
            clan_id
        )
        .execute(&self.db)
        .await?;
        
        // End all active wars
        sqlx::query!(
            "UPDATE clan_wars SET ended_at = NOW() WHERE (clan_a = $1 OR clan_b = $1) AND ended_at IS NULL",
            clan_id
        )
        .execute(&self.db)
        .await?;
        
        // Delete the clan
        sqlx::query!(
            "DELETE FROM clans WHERE id = $1",
            clan_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
}