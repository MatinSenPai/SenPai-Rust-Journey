use taskforge_core::{Job, JobStatus};

/// Formats a `/status` reply: counts by status, then the most recent jobs.
/// Pure formatting logic — zero I/O, zero Telegram/HTTP dependency — which
/// is exactly why it's tested directly here instead of only being
/// exercised by actually running the bot against a live API and a live
/// Telegram chat.
pub fn format_status(jobs: &[Job]) -> String {
    if jobs.is_empty() {
        return "No jobs found.".to_string();
    }

    let mut pending = 0;
    let mut running = 0;
    let mut succeeded = 0;
    let mut retrying = 0;
    let mut failed = 0;
    let mut dead_letter = 0;

    for job in jobs {
        match job.status {
            JobStatus::Pending => pending += 1,
            JobStatus::Running => running += 1,
            JobStatus::Succeeded => succeeded += 1,
            JobStatus::Retrying { .. } => retrying += 1,
            JobStatus::Failed { .. } => failed += 1,
            JobStatus::DeadLetter { .. } => dead_letter += 1,
        }
    }

    let mut out = format!(
        "Jobs ({} shown): {pending} pending, {running} running, {succeeded} succeeded, \
         {retrying} retrying, {failed} failed, {dead_letter} dead-lettered\n\n",
        jobs.len()
    );

    for job in jobs.iter().take(5) {
        out.push_str(&format!(
            "- {} [{}] {:?}\n",
            job.id, job.job_type, job.status
        ));
    }

    out
}

pub fn format_cancel_result(id: &str, result: &Result<(), String>) -> String {
    match result {
        Ok(()) => format!("Cancelled job {id}."),
        Err(error) => format!("Failed to cancel job {id}: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use taskforge_core::JobId;

    fn sample_job(status: JobStatus) -> Job {
        let now = Utc::now();
        Job {
            id: JobId::new(),
            job_type: "send_email".to_string(),
            payload: serde_json::json!({}),
            status,
            attempts: 0,
            max_attempts: 5,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn empty_job_list_reports_no_jobs() {
        assert_eq!(format_status(&[]), "No jobs found.");
    }

    #[test]
    fn counts_each_status_correctly() {
        let jobs = vec![
            sample_job(JobStatus::Pending),
            sample_job(JobStatus::Pending),
            sample_job(JobStatus::Succeeded),
            sample_job(JobStatus::DeadLetter {
                error: "boom".to_string(),
            }),
        ];
        let status = format_status(&jobs);
        assert!(status.contains("2 pending"));
        assert!(status.contains("1 succeeded"));
        assert!(status.contains("1 dead-lettered"));
        assert!(status.contains("0 running"));
    }

    #[test]
    fn cancel_result_formats_success_and_failure() {
        assert_eq!(format_cancel_result("abc", &Ok(())), "Cancelled job abc.");
        assert_eq!(
            format_cancel_result("abc", &Err("already running".to_string())),
            "Failed to cancel job abc: already running"
        );
    }
}
