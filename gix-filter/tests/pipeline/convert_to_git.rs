use crate::driver::apply::driver_with_process;
use crate::eol::convert_to_git::{no_call, no_object_in_index};
use crate::pipeline::pipeline;
use bstr::ByteSlice;
use gix_filter::pipeline::CrlfRoundTripCheck;
use std::io::Read;
use std::path::Path;

#[test]
fn all_stages_mean_streaming_is_impossible() -> gix_testtools::Result {
    let (mut cache, mut pipe) = pipeline("all-filters", || {
        (vec![driver_with_process()], Vec::new(), CrlfRoundTripCheck::Fail)
    })?;

    let mut out = pipe.convert_to_git(
        "➡a\r\n➡b\r\n➡$Id: 2188d1cdee2b93a80084b61af431a49d21bc7cc0$".as_bytes(),
        Path::new("any.txt"),
        |path, attrs| {
            cache
                .at_entry(path, Some(false), |_oid, _buf| -> Result<_, std::convert::Infallible> {
                    unreachable!("index access disabled")
                })
                .expect("cannot fail")
                .matching_attributes(attrs);
        },
        no_object_in_index,
    )?;
    assert!(out.is_changed(), "filters were applied");
    assert!(out.as_read().is_none(), "non-driver filters operate in-memory");
    let buf = out.as_bytes().expect("in-memory operation");
    assert_eq!(buf.as_bstr(), "a\nb\n$Id$", "filters were successfully reversed");
    Ok(())
}

#[test]
fn only_driver_means_streaming_is_possible() -> gix_testtools::Result {
    let (mut cache, mut pipe) = pipeline("driver-only", || {
        (vec![driver_with_process()], Vec::new(), CrlfRoundTripCheck::Skip)
    })?;

    let mut out = pipe.convert_to_git(
        "➡a\r\n➡b\r\n➡$Id: 2188d1cdee2b93a80084b61af431a49d21bc7cc0$".as_bytes(),
        Path::new("subdir/doesnot/matter/any.txt"),
        |path, attrs| {
            cache
                .at_entry(path, Some(false), |_oid, _buf| -> Result<_, std::convert::Infallible> {
                    unreachable!("index access disabled")
                })
                .expect("cannot fail")
                .matching_attributes(attrs);
        },
        no_object_in_index,
    )?;
    assert!(out.is_changed(), "filters were applied");
    assert!(out.as_read().is_some(), "filter-only can be streamed");
    let mut buf = Vec::new();
    out.read_to_end(&mut buf)?;
    assert_eq!(
        buf.as_bstr(),
        "a\r\nb\r\n$Id: 2188d1cdee2b93a80084b61af431a49d21bc7cc0$",
        "one filter was reversed"
    );
    Ok(())
}

#[test]
fn no_filter_means_reader_is_returned_unchanged() -> gix_testtools::Result {
    let (mut cache, mut pipe) = pipeline("no-filters", || (vec![], Vec::new(), CrlfRoundTripCheck::Fail))?;

    let input = "➡a\r\n➡b\r\n➡$Id: 2188d1cdee2b93a80084b61af431a49d21bc7cc0$";
    let mut out = pipe.convert_to_git(
        input.as_bytes(),
        Path::new("other.txt"),
        |path, attrs| {
            cache
                .at_entry(path, Some(false), |_oid, _buf| -> Result<_, std::convert::Infallible> {
                    unreachable!("index access disabled")
                })
                .expect("cannot fail")
                .matching_attributes(attrs);
        },
        no_call,
    )?;
    assert!(!out.is_changed(), "no filter was applied");
    let actual = out
        .as_read()
        .expect("input is unchanged, we get the original stream back");
    let mut buf = Vec::new();
    actual.read_to_end(&mut buf)?;
    assert_eq!(buf.as_bstr(), input, "input is unchanged");
    Ok(())
}
