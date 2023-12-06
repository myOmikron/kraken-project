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

from kraken_sdk.api.attacks_api import AttacksApi


class TestAttacksApi(unittest.TestCase):
    """AttacksApi unit test stubs"""

    def setUp(self) -> None:
        self.api = AttacksApi()

    def tearDown(self) -> None:
        pass

    def test_bruteforce_subdomains(self) -> None:
        """Test case for bruteforce_subdomains

        Bruteforce subdomains through a DNS wordlist attack
        """
        pass

    def test_delete_attack(self) -> None:
        """Test case for delete_attack

        Delete an attack and its results
        """
        pass

    def test_dns_resolution(self) -> None:
        """Test case for dns_resolution

        Perform domain name resolution
        """
        pass

    def test_get_all_attacks(self) -> None:
        """Test case for get_all_attacks

        Retrieve all attacks the user has access to
        """
        pass

    def test_get_attack(self) -> None:
        """Test case for get_attack

        Retrieve an attack by id
        """
        pass

    def test_get_bruteforce_subdomains_results(self) -> None:
        """Test case for get_bruteforce_subdomains_results

        Retrieve a bruteforce subdomains' results by the attack's id
        """
        pass

    def test_get_dns_resolution_results(self) -> None:
        """Test case for get_dns_resolution_results

        Retrieve a dns resolution's results by the attack's id
        """
        pass

    def test_get_host_alive_results(self) -> None:
        """Test case for get_host_alive_results

        Retrieve a host alive's results by the attack's id
        """
        pass

    def test_get_query_certificate_transparency_results(self) -> None:
        """Test case for get_query_certificate_transparency_results

        Retrieve a query certificate transparency's results by the attack's id
        """
        pass

    def test_get_query_unhashed_results(self) -> None:
        """Test case for get_query_unhashed_results

        Retrieve a query dehashed's results by the attack's id
        """
        pass

    def test_get_service_detection_results(self) -> None:
        """Test case for get_service_detection_results

        Retrieve a detect service's results by the attack's id
        """
        pass

    def test_get_tcp_port_scan_results(self) -> None:
        """Test case for get_tcp_port_scan_results

        Retrieve a tcp port scan's results by the attack's id
        """
        pass

    def test_get_workspace_attacks(self) -> None:
        """Test case for get_workspace_attacks

        Query all attacks of a workspace
        """
        pass

    def test_hosts_alive_check(self) -> None:
        """Test case for hosts_alive_check

        Check if hosts are reachable
        """
        pass

    def test_query_certificate_transparency(self) -> None:
        """Test case for query_certificate_transparency

        Query a certificate transparency log collector.
        """
        pass

    def test_query_dehashed(self) -> None:
        """Test case for query_dehashed

        Query the [dehashed](https://dehashed.com/) API.
        """
        pass

    def test_scan_tcp_ports(self) -> None:
        """Test case for scan_tcp_ports

        Start a tcp port scan
        """
        pass

    def test_service_detection(self) -> None:
        """Test case for service_detection

        Perform service detection on a ip and port combination
        """
        pass


if __name__ == '__main__':
    unittest.main()