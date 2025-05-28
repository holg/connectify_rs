use chrono::{DateTime, Duration, NaiveTime, Utc, Weekday};
use chrono_tz::Tz;
use connectify_gcal::logic::calculate_available_slots;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Helper function to create a valid time range
fn create_time_range(start_offset_hours: i64, duration_days: i64) -> (DateTime<Tz>, DateTime<Tz>) {
    let time_zone = Tz::Europe__Zurich;
    let start = Utc::now().with_timezone(&time_zone) + Duration::hours(start_offset_hours);
    let end = start + Duration::days(duration_days);
    (start, end)
}

// Helper function to create a list of busy periods
fn create_busy_periods(
    base_time: DateTime<Tz>,
    count: usize,
    max_duration_hours: i64,
) -> Vec<(DateTime<Tz>, DateTime<Tz>)> {
    let mut busy_periods = Vec::new();
    let mut current_time = base_time;

    for _ in 0..count {
        let start = current_time + Duration::hours(1);
        let end = start + Duration::hours(max_duration_hours.max(1));
        busy_periods.push((start, end));
        current_time = end + Duration::hours(1);
    }

    busy_periods
}

fn benchmark_calculate_available_slots(c: &mut Criterion) {
    // Create a benchmark group for calculate_available_slots
    let mut group = c.benchmark_group("calculate_available_slots");

    // Benchmark with no busy periods
    group.bench_function("no_busy_periods", |b| {
        b.iter(|| {
            let (start, end) = create_time_range(0, 7); // 1 week
            let busy_periods = Vec::new();
            let duration = Duration::minutes(60);
            let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
            let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
            let working_days = [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ];
            let buffer = Duration::minutes(0);
            let step = Duration::minutes(15);

            calculate_available_slots(
                black_box(start),
                black_box(end),
                black_box(&busy_periods),
                black_box(duration),
                black_box(work_start),
                black_box(work_end),
                black_box(&working_days),
                black_box(buffer),
                black_box(step),
            )
        })
    });

    // Benchmark with a few busy periods
    group.bench_function("few_busy_periods", |b| {
        b.iter(|| {
            let (start, end) = create_time_range(0, 7); // 1 week
            let busy_periods = create_busy_periods(start, 5, 2); // 5 busy periods of 2 hours each
            let duration = Duration::minutes(60);
            let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
            let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
            let working_days = [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ];
            let buffer = Duration::minutes(0);
            let step = Duration::minutes(15);
            calculate_available_slots(
                black_box(start),
                black_box(end),
                black_box(&busy_periods),
                black_box(duration),
                black_box(work_start),
                black_box(work_end),
                black_box(&working_days),
                black_box(buffer),
                black_box(step),
            )
        })
    });

    // Benchmark with many busy periods
    group.bench_function("many_busy_periods", |b| {
        b.iter(|| {
            let (start, end) = create_time_range(0, 7); // 1 week
            let busy_periods = create_busy_periods(start, 20, 2); // 20 busy periods of 2 hours each
            let duration = Duration::minutes(60);
            let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
            let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
            let working_days = [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ];
            let buffer = Duration::minutes(0);
            let step = Duration::minutes(15);

            calculate_available_slots(
                black_box(start),
                black_box(end),
                black_box(&busy_periods),
                black_box(duration),
                black_box(work_start),
                black_box(work_end),
                black_box(&working_days),
                black_box(buffer),
                black_box(step),
            )
        })
    });

    // Benchmark with buffer time
    group.bench_function("with_buffer", |b| {
        b.iter(|| {
            let (start, end) = create_time_range(0, 7); // 1 week
            let busy_periods = create_busy_periods(start, 5, 2); // 5 busy periods of 2 hours each
            let duration = Duration::minutes(60);
            let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
            let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
            let working_days = [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ];
            let buffer = Duration::minutes(15); // 15-minute buffer
            let step = Duration::minutes(15);

            calculate_available_slots(
                black_box(start),
                black_box(end),
                black_box(&busy_periods),
                black_box(duration),
                black_box(work_start),
                black_box(work_end),
                black_box(&working_days),
                black_box(buffer),
                black_box(step),
            )
        })
    });

    // Benchmark with longer duration
    group.bench_function("longer_duration", |b| {
        b.iter(|| {
            let (start, end) = create_time_range(0, 7); // 1 week
            let busy_periods = create_busy_periods(start, 5, 2); // 5 busy periods of 2 hours each
            let duration = Duration::minutes(120); // 2-hour duration
            let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
            let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
            let working_days = [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ];
            let buffer = Duration::minutes(0);
            let step = Duration::minutes(15);

            calculate_available_slots(
                black_box(start),
                black_box(end),
                black_box(&busy_periods),
                black_box(duration),
                black_box(work_start),
                black_box(work_end),
                black_box(&working_days),
                black_box(buffer),
                black_box(step),
            )
        })
    });

    // Benchmark with longer time range
    group.bench_function("longer_time_range", |b| {
        b.iter(|| {
            let (start, end) = create_time_range(0, 30); // 1 month
            let busy_periods = create_busy_periods(start, 5, 2); // 5 busy periods of 2 hours each
            let duration = Duration::minutes(60);
            let work_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
            let work_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
            let working_days = [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ];
            let buffer = Duration::minutes(0);
            let step = Duration::minutes(15);

            calculate_available_slots(
                black_box(start),
                black_box(end),
                black_box(&busy_periods),
                black_box(duration),
                black_box(work_start),
                black_box(work_end),
                black_box(&working_days),
                black_box(buffer),
                black_box(step),
            )
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_calculate_available_slots);
criterion_main!(benches);
