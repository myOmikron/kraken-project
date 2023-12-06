# coding: utf-8

"""
    kraken

    The core component of kraken-project

    The version of the OpenAPI document: 0.1.0
    Contact: git@omikron.dev
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


import unittest
import datetime

from kraken_sdk.models.simple_tcp_port_scan_result import SimpleTcpPortScanResult

class TestSimpleTcpPortScanResult(unittest.TestCase):
    """SimpleTcpPortScanResult unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> SimpleTcpPortScanResult:
        """Test SimpleTcpPortScanResult
            include_option is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `SimpleTcpPortScanResult`
        """
        model = SimpleTcpPortScanResult()
        if include_optional:
            return SimpleTcpPortScanResult(
                uuid = '',
                attack = '',
                created_at = datetime.datetime.strptime('2013-10-20 19:20:30.00', '%Y-%m-%d %H:%M:%S.%f'),
                address = '127.0.0.1',
                port = 0
            )
        else:
            return SimpleTcpPortScanResult(
                uuid = '',
                attack = '',
                created_at = datetime.datetime.strptime('2013-10-20 19:20:30.00', '%Y-%m-%d %H:%M:%S.%f'),
                address = '127.0.0.1',
                port = 0,
        )
        """

    def testSimpleTcpPortScanResult(self):
        """Test SimpleTcpPortScanResult"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()