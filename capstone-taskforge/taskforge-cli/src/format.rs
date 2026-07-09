use taskforge_core::Job;

/// One line of terminal output for a job. Pure formatting — zero I/O —
/// tested directly here rather than only via a real end-to-end CLI run.
pub fn format_job_line(job: &Job) -> String {
    format!("{}  {:<20}  {:?}", job.id, job.job_type, job.status)
}

pub fn format_job_list(jobs: &[Job]) -> String {
    if jobs.is_empty() {
        return "No jobs found.".to_string();
    }
    jobs.iter()
        .map(format_job_line)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use taskforge_core::{JobId, JobStatus};

    fn sample_job() -> Job {
        let now = Utc::now();
        Job {
            id: JobId::new(),
            job_type: "send_email".to_string(),
            payload: serde_json::json!({}),
            status: JobStatus::Pending,
            attempts: 0,
            max_attempts: 5,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn formats_a_single_job_line() {
        let job = sample_job();
        let line = format_job_line(&job);
        assert!(line.contains(&job.id.to_string()));
        assert!(line.contains("send_email"));
        assert!(line.contains("Pending"));
    }

    #[test]
    fn empty_list_reports_no_jobs() {
        assert_eq!(format_job_list(&[]), "No jobs found.");
    }

    #[test]
    fn list_joins_lines_for_each_job() {
        let jobs = vec![sample_job(), sample_job()];
        let output = format_job_list(&jobs);
        assert_eq!(output.lines().count(), 2);
    }
}
