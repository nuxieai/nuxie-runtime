//! Browser-owned bounded surface acquisition and recovery policy.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SurfaceAcquisitionFailure {
    Timeout,
    Occluded,
    Outdated,
    Lost,
    Validation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SurfaceRecoveryAction {
    ReconfigureAndRetry,
    RecreateAndRetry,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum SurfaceRecoveryError<E> {
    Acquisition {
        failure: SurfaceAcquisitionFailure,
        recovery: Option<SurfaceRecoveryAction>,
    },
    Recovery(E),
}

pub(super) fn acquire_surface_texture<T, E>(
    mut acquire: impl FnMut() -> Result<T, SurfaceAcquisitionFailure>,
    mut recover: impl FnMut(SurfaceRecoveryAction) -> Result<(), E>,
) -> Result<T, SurfaceRecoveryError<E>> {
    let failure = match acquire() {
        Ok(texture) => return Ok(texture),
        Err(failure) => failure,
    };
    let Some(recovery) = surface_recovery_action(failure) else {
        return Err(SurfaceRecoveryError::Acquisition {
            failure,
            recovery: None,
        });
    };
    recover(recovery).map_err(SurfaceRecoveryError::Recovery)?;
    acquire().map_err(|failure| SurfaceRecoveryError::Acquisition {
        failure,
        recovery: Some(recovery),
    })
}

fn surface_recovery_action(failure: SurfaceAcquisitionFailure) -> Option<SurfaceRecoveryAction> {
    match failure {
        SurfaceAcquisitionFailure::Outdated => Some(SurfaceRecoveryAction::ReconfigureAndRetry),
        SurfaceAcquisitionFailure::Lost => Some(SurfaceRecoveryAction::RecreateAndRetry),
        SurfaceAcquisitionFailure::Timeout
        | SurfaceAcquisitionFailure::Occluded
        | SurfaceAcquisitionFailure::Validation => None,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::convert::Infallible;

    use super::{
        SurfaceAcquisitionFailure, SurfaceRecoveryAction, SurfaceRecoveryError,
        acquire_surface_texture, surface_recovery_action,
    };

    #[test]
    fn outdated_surface_is_reconfigured_then_acquired_again() {
        let acquisitions = Cell::new(0);
        let mut results = [
            Err(SurfaceAcquisitionFailure::Outdated),
            Ok("surface-texture"),
        ]
        .into_iter();
        let recoveries = RefCell::new(Vec::new());

        let result: Result<&str, SurfaceRecoveryError<Infallible>> = acquire_surface_texture(
            || {
                acquisitions.set(acquisitions.get() + 1);
                results.next().expect("driver acquires exactly twice")
            },
            |action| {
                recoveries.borrow_mut().push(action);
                Ok(())
            },
        );

        assert_eq!(result, Ok("surface-texture"));
        assert_eq!(acquisitions.get(), 2);
        assert_eq!(
            recoveries.into_inner(),
            vec![SurfaceRecoveryAction::ReconfigureAndRetry]
        );
    }

    #[test]
    fn lost_surface_is_recreated_then_acquired_again() {
        let acquisitions = Cell::new(0);
        let mut results = [Err(SurfaceAcquisitionFailure::Lost), Ok("surface-texture")].into_iter();
        let recoveries = RefCell::new(Vec::new());

        let result: Result<&str, SurfaceRecoveryError<Infallible>> = acquire_surface_texture(
            || {
                acquisitions.set(acquisitions.get() + 1);
                results.next().expect("driver acquires exactly twice")
            },
            |action| {
                recoveries.borrow_mut().push(action);
                Ok(())
            },
        );

        assert_eq!(result, Ok("surface-texture"));
        assert_eq!(acquisitions.get(), 2);
        assert_eq!(
            recoveries.into_inner(),
            vec![SurfaceRecoveryAction::RecreateAndRetry]
        );
    }

    #[test]
    fn second_failure_returns_typed_error_without_a_third_acquisition() {
        for (failure, recovery) in [
            (
                SurfaceAcquisitionFailure::Outdated,
                SurfaceRecoveryAction::ReconfigureAndRetry,
            ),
            (
                SurfaceAcquisitionFailure::Lost,
                SurfaceRecoveryAction::RecreateAndRetry,
            ),
        ] {
            let acquisitions = Cell::new(0);
            let mut results = [Err(failure), Err(failure)].into_iter();
            let recoveries = RefCell::new(Vec::new());

            let result: Result<(), SurfaceRecoveryError<Infallible>> = acquire_surface_texture(
                || {
                    acquisitions.set(acquisitions.get() + 1);
                    results.next().expect("driver acquires exactly twice")
                },
                |action| {
                    recoveries.borrow_mut().push(action);
                    Ok(())
                },
            );

            assert_eq!(
                result,
                Err(SurfaceRecoveryError::Acquisition {
                    failure,
                    recovery: Some(recovery),
                })
            );
            assert_eq!(acquisitions.get(), 2);
            assert_eq!(recoveries.into_inner(), vec![recovery]);
        }
    }

    #[test]
    fn failed_recovery_is_returned_without_a_second_acquisition() {
        let acquisitions = Cell::new(0);
        let result: Result<(), SurfaceRecoveryError<&str>> = acquire_surface_texture(
            || {
                acquisitions.set(acquisitions.get() + 1);
                Err(SurfaceAcquisitionFailure::Lost)
            },
            |action| {
                assert_eq!(action, SurfaceRecoveryAction::RecreateAndRetry);
                Err("surface recreation failed")
            },
        );

        assert_eq!(
            result,
            Err(SurfaceRecoveryError::Recovery("surface recreation failed"))
        );
        assert_eq!(acquisitions.get(), 1);
    }

    #[test]
    fn transient_and_validation_failures_do_not_reconfigure_or_recreate() {
        for failure in [
            SurfaceAcquisitionFailure::Timeout,
            SurfaceAcquisitionFailure::Occluded,
            SurfaceAcquisitionFailure::Validation,
        ] {
            assert_eq!(surface_recovery_action(failure), None);
        }
    }
}
