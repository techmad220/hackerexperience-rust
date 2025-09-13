use sqlx::{MySql, Pool};
use he_core::{Hardware, HardwareInfo, UserId, HardwareId, HeResult, HeError};

// Hardware repository - replaces PHP HardwareVPC.class.php database methods
pub struct HardwareRepository {
    pool: Pool<MySql>,
}

impl HardwareRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
    
    // Create hardware entry - new server
    pub async fn create_hardware(&self, mut hardware: Hardware) -> HeResult<Hardware> {
        let result = sqlx::query!(
            r#"
            INSERT INTO hardware (user_id, name, cpu, ram, hdd, net, is_npc)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            hardware.user_id,
            format!("Server-{}", hardware.user_id), // Default name
            hardware.cpu,
            hardware.ram,
            hardware.hdd, 
            hardware.net,
            hardware.is_npc
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        hardware.id = result.last_insert_id() as HardwareId;
        Ok(hardware)
    }
    
    // Get hardware info for user - equivalent to PHP getHardwareInfo method
    pub async fn get_hardware_info(&self, user_id: UserId, is_npc: bool) -> HeResult<HardwareInfo> {
        let npc_flag = if is_npc { 1 } else { 0 };
        
        let row = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_pcs,
                SUM(cpu) as total_cpu,
                SUM(hdd) as total_hdd,
                SUM(ram) as total_ram,
                MAX(net) as network_speed
            FROM hardware 
            WHERE user_id = ? AND is_npc = ?
            "#,
            user_id,
            npc_flag
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        let info = HardwareInfo {
            total_pcs: row.total_pcs as i32,
            total_cpu: row.total_cpu.unwrap_or(0.0) as i32,
            total_ram: row.total_ram.unwrap_or(0.0) as i32,
            total_hdd: row.total_hdd.unwrap_or(0.0) as i32,
            network_speed: row.network_speed.unwrap_or(0.0) as i32,
        };
        
        Ok(info)
    }
    
    // Get all hardware for a user
    pub async fn get_user_hardware(&self, user_id: UserId) -> HeResult<Vec<Hardware>> {
        let rows = sqlx::query!(
            "SELECT * FROM hardware WHERE user_id = ? AND is_npc = 0",
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        let mut hardware_list = Vec::new();
        for row in rows {
            let hardware = Hardware {
                id: row.server_id as HardwareId,
                user_id: row.user_id as UserId,
                ram: row.ram as i32,
                cpu: row.cpu as i32,
                hdd: row.hdd as i32,
                net: row.net as i32,
                is_npc: row.is_npc != 0,
            };
            hardware_list.push(hardware);
        }
        
        Ok(hardware_list)
    }
    
    // Get NPC hardware
    pub async fn get_npc_hardware(&self, user_id: UserId) -> HeResult<Vec<Hardware>> {
        let rows = sqlx::query!(
            "SELECT * FROM hardware WHERE user_id = ? AND is_npc = 1",
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        let mut hardware_list = Vec::new();
        for row in rows {
            let hardware = Hardware {
                id: row.server_id as HardwareId,
                user_id: row.user_id as UserId,
                ram: row.ram as i32,
                cpu: row.cpu as i32,
                hdd: row.hdd as i32,
                net: row.net as i32,
                is_npc: row.is_npc != 0,
            };
            hardware_list.push(hardware);
        }
        
        Ok(hardware_list)
    }
    
    // Update hardware specs
    pub async fn update_hardware(&self, hardware: &Hardware) -> HeResult<()> {
        sqlx::query!(
            r#"
            UPDATE hardware 
            SET cpu = ?, ram = ?, hdd = ?, net = ?
            WHERE server_id = ?
            "#,
            hardware.cpu,
            hardware.ram,
            hardware.hdd,
            hardware.net,
            hardware.id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(())
    }
    
    // Delete hardware
    pub async fn delete_hardware(&self, hardware_id: HardwareId) -> HeResult<()> {
        sqlx::query!(
            "DELETE FROM hardware WHERE server_id = ?",
            hardware_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| HeError::Database(e.into()))?;
        
        Ok(())
    }
}