use crate::core::{
    dff,
    option::{pack, unpack},
};

use rhdl::prelude::*;

#[derive(Debug, Clone, Synchronous, SynchronousDQ)]
/// The PipeReducer Core
///
/// This core takes a stream of `[T; N]`, and produces
/// a stream of `T`, reading out the input stream in
/// index order (`0, 1, 2..`).  
pub struct PipeReducer<M: BitWidth, T: Digital, const N: usize>
where
    [T; N]: Default,
{
    store: dff::DFF<Option<[T; N]>>,
    count: dff::DFF<Bits<M>>,
    marker: std::marker::PhantomData<T>,
}

impl<M: BitWidth, T: Digital, const N: usize> Default for PipeReducer<M, T, N>
where
    [T; N]: Default,
{
    fn default() -> Self {
        assert!((1 << M::BITS) == N, "Expect that the bitwidth of the counter is exactly sufficient to count the elements in the array.  I.e., (1 << M) == N");
        Self {
            store: dff::DFF::new(None),
            count: dff::DFF::new(bits(0)),
            marker: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, PartialEq, Digital)]
/// Inputs for the [PipeReducer] core
pub struct In<T: Digital, const N: usize> {
    /// Input data elements that need to be reduced
    pub data: Option<[T; N]>,
    /// Input ready flag from downstream
    pub ready: bool,
}

#[derive(Debug, PartialEq, Digital)]
/// Outputs from the [PipeReducer] core
pub struct Out<T: Digital> {
    /// Output data elements from the input array
    pub data: Option<T>,
    /// Output ready flag to upstream
    pub ready: bool,
}

impl<M: BitWidth, T: Digital, const N: usize> SynchronousIO for PipeReducer<M, T, N>
where
    [T; N]: Default,
{
    type I = In<T, N>;
    type O = Out<T>;
    type Kernel = kernel<M, T, N>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<M: BitWidth, T: Digital, const N: usize>(
    _cr: ClockReset,
    i: In<T, N>,
    q: Q<M, T, N>,
) -> (Out<T>, D<M, T, N>)
where
    [T; N]: Default,
{
    // Extract the tag from the input
    let (tag, data) = unpack::<[T; N]>(q.store);
    let mut out = Out::<T>::dont_care();
    // The output value is the input data selected with
    // the tag copied from the input
    out.data = pack::<T>(tag, data[q.count]);
    // This boolean indicates the pipeline will advance
    let will_run = tag && i.ready;
    let mut d = D::<M, T, N>::dont_care();
    // If we advance, then roll the counter
    d.count = q.count + if will_run { 1 } else { 0 };
    // The store DFF will normally hold state unless
    // it is empty.
    d.store = q.store;
    // The two cases where it will be open to the input
    // bus is if it is empty/None, or if we will finish
    // with the contents.
    out.ready = false;
    if !tag || (will_run && q.count == (N as u128 - 1)) {
        d.store = i.data;
        out.ready = true;
    }
    (out, d)
}

#[cfg(test)]
mod tests {

    use crate::rng::xorshift::XorShift128;

    use super::*;

    fn mk_array<T, const N: usize>(t: &mut impl Iterator<Item = T>) -> [T; N]
    where
        [T; N]: Default,
    {
        let mut ret = <[T; N] as Default>::default();
        (0..N).for_each(|ndx| {
            ret[ndx] = t.next().unwrap();
        });
        ret
    }

    #[test]
    fn test_operation() -> miette::Result<()> {
        type UUT = PipeReducer<U2, b4, 4>;
        let uut = UUT::default();
        let mut need_reset = true;
        let mut source_rng = XorShift128::default().map(|x| bits((x & 0xF) as u128));
        let mut dest_rng = source_rng.clone();
        let mut latched_input: Option<[b4; 4]> = None;
        let vcd = uut
            .run_fn(
                move |out| {
                    if need_reset {
                        need_reset = false;
                        return Some(rhdl::core::sim::ResetOrData::Reset);
                    }
                    let mut input = super::In::<b4, 4>::dont_care();
                    // Downstream is likely to run
                    let want_to_pause = rand::random::<u8>() > 200;
                    input.ready = !want_to_pause;
                    // Decide if the producer will generate a new data item
                    let willing_to_send = rand::random::<u8>() < 200;
                    if out.ready {
                        // The pipeline wants more data
                        if willing_to_send {
                            latched_input = Some(mk_array(&mut source_rng));
                        } else {
                            latched_input = None;
                        }
                    }
                    input.data = latched_input;
                    Some(rhdl::core::sim::ResetOrData::Data(input))
                },
                100,
            )
            .take_while(|t| t.time < 10_000)
            .collect::<Vcd>();
        vcd.dump_to_file(&std::path::PathBuf::from("jnk_reducer.vcd"))
            .unwrap();
        Ok(())
    }
}
