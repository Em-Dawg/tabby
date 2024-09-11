use sqlx::query;

use crate::DbConn;

impl DbConn {
    pub async fn allow_read_source(&self, user_id: i64, source_id: &str) -> anyhow::Result<bool> {
        let is_public_source = query!(
            "select id from source_id_read_access_policies where source_id = ?",
            source_id
        )
        .fetch_optional(&self.pool)
        .await?
        .is_none();

        if is_public_source {
            return Ok(true);
        }

        let row = query!(
            r#"
SELECT user_id
FROM user_groups
    INNER JOIN user_group_memberships ON user_group_memberships.user_group_id = user_groups.id
    INNER JOIN source_id_read_access_policies ON source_id_read_access_policies.user_group_id = user_groups.id
WHERE user_group_memberships.user_id = ? AND source_id = ?
        "#,
            user_id,
            source_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    pub async fn upsert_source_id_read_access_policy(
        &self,
        source_id: &str,
        user_group_id: i64,
    ) -> anyhow::Result<()> {
        query!(
            r#"
INSERT INTO source_id_read_access_policies (source_id, user_group_id) VALUES (?, ?)
ON CONFLICT (source_id, user_group_id) DO UPDATE
  SET updated_at = DATETIME("now")
        "#,
            source_id,
            user_group_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_source_id_read_access_policy(
        &self,
        source_id: &str,
        user_group_id: i64,
    ) -> anyhow::Result<()> {
        let rows_deleted = query!(
            r#"
DELETE FROM source_id_read_access_policies WHERE source_id = ? AND user_group_id = ?
        "#,
            source_id,
            user_group_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        if rows_deleted == 1 {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "source_id_read_access_policy doesn't exist",
            ))
        }
    }
}