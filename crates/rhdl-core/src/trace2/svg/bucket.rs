use crate::{TypedBits, trace2::svg::color::TraceColor};

/// A bucket representing a contiguous time span where the value is the same.
/// Stores the data as [TypedBits] along with start and end time.  Also
/// stores the color to be used when rendering this bucket.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Bucket {
    pub(crate) start: u64,
    pub(crate) end: u64,
    pub(crate) data: TypedBits,
    pub(crate) color: TraceColor,
}

pub(crate) fn bucketize(
    data: impl IntoIterator<Item = (u64, Option<TypedBits>)>,
    color: TraceColor,
) -> Box<[Bucket]> {
    let mut buckets = Vec::new();
    let mut last_time = !0;
    let mut last_data = None;
    let mut start_time = !0;
    for (time, data) in data.into_iter() {
        if last_time == !0 {
            last_time = time;
            start_time = time;
            last_data = data.clone();
        } else {
            if !last_data.eq(&data) {
                if let Some(data) = last_data
                    && start_time != time
                {
                    buckets.push(Bucket {
                        start: start_time,
                        end: time,
                        data: data.clone(),
                        color,
                    });
                }
                start_time = time;
                last_data = data.clone();
            }
            last_time = time;
        }
    }
    if let Some(data) = last_data {
        buckets.push(Bucket {
            start: start_time,
            end: last_time,
            color,
            data,
        });
    }
    buckets.into()
}
