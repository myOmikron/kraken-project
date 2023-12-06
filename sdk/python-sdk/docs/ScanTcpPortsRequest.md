# ScanTcpPortsRequest

The settings to configure a tcp port scan

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**leech_uuid** | **str** | The leech to use  Leave empty to use a random leech | [optional] 
**targets** | **List[str]** | The ip addresses / networks to scan | 
**ports** | [**List[PortOrRange]**](PortOrRange.md) | List of single ports and port ranges  If no values are supplied, 1-65535 is used as default | [optional] 
**retry_interval** | **int** | The interval that should be wait between retries on a port.  The interval is specified in milliseconds. | 
**max_retries** | **int** | The number of times the connection should be retried if it failed. | 
**timeout** | **int** | The time to wait until a connection is considered failed.  The timeout is specified in milliseconds. | 
**concurrent_limit** | **int** | The concurrent task limit | 
**skip_icmp_check** | **bool** | Skips the initial icmp check.  All hosts are assumed to be reachable | 
**workspace_uuid** | **str** | The workspace to execute the attack in | 

## Example

```python
from kraken_sdk.models.scan_tcp_ports_request import ScanTcpPortsRequest

# TODO update the JSON string below
json = "{}"
# create an instance of ScanTcpPortsRequest from a JSON string
scan_tcp_ports_request_instance = ScanTcpPortsRequest.from_json(json)
# print the JSON string representation of the object
print ScanTcpPortsRequest.to_json()

# convert the object into a dict
scan_tcp_ports_request_dict = scan_tcp_ports_request_instance.to_dict()
# create an instance of ScanTcpPortsRequest from a dict
scan_tcp_ports_request_form_dict = scan_tcp_ports_request.from_dict(scan_tcp_ports_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


