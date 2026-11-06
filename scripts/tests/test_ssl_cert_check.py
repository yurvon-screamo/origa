"""Unit tests for scripts/ssl_cert_check.py (incident 2026-11-06).

Railway silently skipped the scheduled renewal of two custom-domain
certificates while ``railway domain status`` reported
``CERTIFICATE_STATUS_TYPE_ISSUE_FAILED`` with ``retryable: true``. The
parsers below pin the exact CLI/RDAP/openssl text shapes the checker
relies on; unknown inputs must degrade to ``None`` (probe error), never
invent an expiry. No network, no CLI calls.
"""

from __future__ import annotations

from datetime import UTC, datetime

from ssl_cert_check import (
    RailwayStatus,
    days_left,
    parse_openssl_enddate,
    parse_railway_status,
    parse_rdap_events,
)

NOW = datetime(2026, 11, 6, 2, 0, 0, tzinfo=UTC).timestamp()


class TestOpensslEnddate:
    def test_parses_double_space_day(self):
        text = "notAfter=Nov  4 17:49:24 2026 GMT\n"
        dt = parse_openssl_enddate(text)
        assert dt == datetime(2026, 11, 4, 17, 49, 24, tzinfo=UTC)

    def test_parses_single_digit_day_with_space(self):
        text = "notAfter=Apr  5 23:59:59 2027 GMT\n"
        assert parse_openssl_enddate(text).year == 2027

    def test_missing_field_returns_none(self):
        assert parse_openssl_enddate("notBefore=Jan 1 00:00:00 2027 GMT\n") is None

    def test_garbage_returns_none(self):
        assert parse_openssl_enddate("notAfter=yesterday\n") is None


class TestDaysLeft:
    def test_future(self):
        exp = datetime(2026, 12, 29, tzinfo=UTC)
        left = days_left(exp, NOW)
        assert 52 < left < 54

    def test_expired_is_negative(self):
        exp = datetime(2026, 11, 4, tzinfo=UTC)
        assert days_left(exp, NOW) < 0

    def test_none_passthrough(self):
        assert days_left(None, NOW) is None


class TestRailwayStatus:
    def test_failed_retryable(self):
        report = (
            "Domain status:\n"
            "  URL: https://app.origa.uwuwu.net\n"
            "  Sync status: ACTIVE\n"
            "  Verified: yes\n"
            "  Certificate status: CERTIFICATE_STATUS_TYPE_ISSUE_FAILED\n"
            "  Certificate detail: CERTIFICATE_STATUS_TYPE_DETAILED_FAILED\n"
            "  Certificate error: An internal error occurred. Please retry or contact support.\n"
            "  Certificate error type: CERTIFICATE_ERROR_TYPE_INTERNAL\n"
            "  Certificate retryable: true\n"
        )
        parsed = parse_railway_status(report)
        assert parsed.cert_status == "CERTIFICATE_STATUS_TYPE_ISSUE_FAILED"
        assert parsed.retryable is True
        assert parsed.verified is True
        assert parsed.issue_failed_retryable is True
        assert parsed.issuing is False

    def test_valid_not_retryable(self):
        report = (
            "  Certificate status: CERTIFICATE_STATUS_TYPE_VALID\n"
            "  Certificate retryable: false\n"
        )
        parsed = parse_railway_status(report)
        assert parsed.issue_failed_retryable is False

    def test_issuing(self):
        report = (
            "  Certificate status: CERTIFICATE_STATUS_TYPE_ISSUING\n"
            "  Certificate detail: CERTIFICATE_STATUS_TYPE_DETAILED_POLLING_AUTHORIZATIONS\n"
        )
        parsed = parse_railway_status(report)
        assert parsed.issuing is True
        assert parsed.issue_failed_retryable is False

    def test_empty_report_is_inert(self):
        parsed = parse_railway_status("Domain status:\n  Sync status: ACTIVE\n")
        assert parsed.cert_status == ""
        assert parsed.issue_failed_retryable is False
        assert parsed.issuing is False

    def test_default_instance_is_inert(self):
        inert = RailwayStatus()
        assert inert.issue_failed_retryable is False


class TestRdapEvents:
    def test_expiration_event(self):
        events = [
            {"eventAction": "registration", "eventDate": "2004-01-01T00:00:00Z"},
            {"eventAction": "expiration", "eventDate": "2026-11-23T00:00:00Z"},
        ]
        dt = parse_rdap_events(events)
        assert dt == datetime(2026, 11, 23, tzinfo=UTC)

    def test_no_expiration_returns_none(self):
        assert parse_rdap_events([{"eventAction": "last changed",
                                   "eventDate": "2026-01-01T00:00:00Z"}]) is None

    def test_empty_events_returns_none(self):
        assert parse_rdap_events([]) is None

    def test_bad_date_returns_none(self):
        assert parse_rdap_events([{"eventAction": "expiration",
                                   "eventDate": "soon"}]) is None

    def test_naive_date_gets_utc(self):
        dt = parse_rdap_events([{"eventAction": "expiration",
                                 "eventDate": "2026-11-23T00:00:00"}])
        assert dt is not None
        assert dt.tzinfo is UTC

    def test_offset_date_preserved(self):
        dt = parse_rdap_events([{"eventAction": "expiration",
                                 "eventDate": "2026-11-23T09:00:00+09:00"}])
        assert dt.utcoffset().total_seconds() == 9 * 3600
