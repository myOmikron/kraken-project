# SearchResultEntry

Dynamic result

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**host_entry** | [**SimpleHost**](SimpleHost.md) |  | 
**service_entry** | [**SimpleService**](SimpleService.md) |  | 
**port_entry** | [**SimplePort**](SimplePort.md) |  | 
**domain_entry** | [**SimpleDomain**](SimpleDomain.md) |  | 
**dns_record_result_entry** | [**SimpleDnsResolutionResult**](SimpleDnsResolutionResult.md) |  | 
**tcp_port_scan_result_entry** | [**SimpleTcpPortScanResult**](SimpleTcpPortScanResult.md) |  | 
**dehashed_query_result_entry** | [**SimpleQueryUnhashedResult**](SimpleQueryUnhashedResult.md) |  | 
**certificate_transparency_result_entry** | [**FullQueryCertificateTransparencyResult**](FullQueryCertificateTransparencyResult.md) |  | 
**host_alive_result** | [**SimpleHostAliveResult**](SimpleHostAliveResult.md) |  | 
**service_detection_result** | [**FullServiceDetectionResult**](FullServiceDetectionResult.md) |  | 

## Example

```python
from kraken_sdk.models.search_result_entry import SearchResultEntry

# TODO update the JSON string below
json = "{}"
# create an instance of SearchResultEntry from a JSON string
search_result_entry_instance = SearchResultEntry.from_json(json)
# print the JSON string representation of the object
print SearchResultEntry.to_json()

# convert the object into a dict
search_result_entry_dict = search_result_entry_instance.to_dict()
# create an instance of SearchResultEntry from a dict
search_result_entry_form_dict = search_result_entry.from_dict(search_result_entry_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


