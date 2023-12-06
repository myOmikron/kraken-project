# kraken_sdk.AttacksApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**bruteforce_subdomains**](AttacksApi.md#bruteforce_subdomains) | **POST** /api/v1/attacks/bruteforceSubdomains | Bruteforce subdomains through a DNS wordlist attack
[**delete_attack**](AttacksApi.md#delete_attack) | **DELETE** /api/v1/attacks/{uuid} | Delete an attack and its results
[**dns_resolution**](AttacksApi.md#dns_resolution) | **POST** /api/v1/attacks/dnsResolution | Perform domain name resolution
[**get_all_attacks**](AttacksApi.md#get_all_attacks) | **GET** /api/v1/attacks | Retrieve all attacks the user has access to
[**get_attack**](AttacksApi.md#get_attack) | **GET** /api/v1/attacks/{uuid} | Retrieve an attack by id
[**get_bruteforce_subdomains_results**](AttacksApi.md#get_bruteforce_subdomains_results) | **GET** /api/v1/attacks/{uuid}/bruteforceSubdomainsResults | Retrieve a bruteforce subdomains&#39; results by the attack&#39;s id
[**get_dns_resolution_results**](AttacksApi.md#get_dns_resolution_results) | **GET** /api/v1/attacks/{uuid}/dnsResolutionResults | Retrieve a dns resolution&#39;s results by the attack&#39;s id
[**get_host_alive_results**](AttacksApi.md#get_host_alive_results) | **GET** /api/v1/attacks/{uuid}/hostAliveResults | Retrieve a host alive&#39;s results by the attack&#39;s id
[**get_query_certificate_transparency_results**](AttacksApi.md#get_query_certificate_transparency_results) | **GET** /api/v1/attacks/{uuid}/queryCertificateTransparencyResults | Retrieve a query certificate transparency&#39;s results by the attack&#39;s id
[**get_query_unhashed_results**](AttacksApi.md#get_query_unhashed_results) | **GET** /api/v1/attacks/{uuid}/queryUnhashedResults | Retrieve a query dehashed&#39;s results by the attack&#39;s id
[**get_service_detection_results**](AttacksApi.md#get_service_detection_results) | **GET** /api/v1/attacks/{uuid}/serviceDetectionResults | Retrieve a detect service&#39;s results by the attack&#39;s id
[**get_tcp_port_scan_results**](AttacksApi.md#get_tcp_port_scan_results) | **GET** /api/v1/attacks/{uuid}/tcpPortScanResults | Retrieve a tcp port scan&#39;s results by the attack&#39;s id
[**get_workspace_attacks**](AttacksApi.md#get_workspace_attacks) | **GET** /api/v1/workspaces/{uuid}/attacks | Query all attacks of a workspace
[**hosts_alive_check**](AttacksApi.md#hosts_alive_check) | **POST** /api/v1/attacks/hostsAlive | Check if hosts are reachable
[**query_certificate_transparency**](AttacksApi.md#query_certificate_transparency) | **POST** /api/v1/attacks/queryCertificateTransparency | Query a certificate transparency log collector.
[**query_dehashed**](AttacksApi.md#query_dehashed) | **POST** /api/v1/attacks/queryDehashed | Query the [dehashed](https://dehashed.com/) API.
[**scan_tcp_ports**](AttacksApi.md#scan_tcp_ports) | **POST** /api/v1/attacks/scanTcpPorts | Start a tcp port scan
[**service_detection**](AttacksApi.md#service_detection) | **POST** /api/v1/attacks/serviceDetection | Perform service detection on a ip and port combination


# **bruteforce_subdomains**
> UuidResponse bruteforce_subdomains(bruteforce_subdomains_request)

Bruteforce subdomains through a DNS wordlist attack

Bruteforce subdomains through a DNS wordlist attack  Enumerate possible subdomains by querying a DNS server with constructed domains. See [OWASP](https://owasp.org/www-community/attacks/Brute_force_attack) for further information.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.bruteforce_subdomains_request import BruteforceSubdomainsRequest
from kraken_sdk.models.uuid_response import UuidResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    bruteforce_subdomains_request = kraken_sdk.BruteforceSubdomainsRequest() # BruteforceSubdomainsRequest | 

    try:
        # Bruteforce subdomains through a DNS wordlist attack
        api_response = await api_instance.bruteforce_subdomains(bruteforce_subdomains_request)
        print("The response of AttacksApi->bruteforce_subdomains:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->bruteforce_subdomains: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **bruteforce_subdomains_request** | [**BruteforceSubdomainsRequest**](BruteforceSubdomainsRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Attack scheduled |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **delete_attack**
> delete_attack(uuid)

Delete an attack and its results

Delete an attack and its results

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Delete an attack and its results
        await api_instance.delete_attack(uuid)
    except Exception as e:
        print("Exception when calling AttacksApi->delete_attack: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Attack was deleted |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **dns_resolution**
> UuidResponse dns_resolution(dns_resolution_request)

Perform domain name resolution

Perform domain name resolution

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.dns_resolution_request import DnsResolutionRequest
from kraken_sdk.models.uuid_response import UuidResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    dns_resolution_request = kraken_sdk.DnsResolutionRequest() # DnsResolutionRequest | 

    try:
        # Perform domain name resolution
        api_response = await api_instance.dns_resolution(dns_resolution_request)
        print("The response of AttacksApi->dns_resolution:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->dns_resolution: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **dns_resolution_request** | [**DnsResolutionRequest**](DnsResolutionRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**202** | Attack scheduled |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_all_attacks**
> ListAttacks get_all_attacks()

Retrieve all attacks the user has access to

Retrieve all attacks the user has access to

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.list_attacks import ListAttacks
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)

    try:
        # Retrieve all attacks the user has access to
        api_response = await api_instance.get_all_attacks()
        print("The response of AttacksApi->get_all_attacks:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->get_all_attacks: %s\n" % e)
```



### Parameters
This endpoint does not need any parameter.

### Return type

[**ListAttacks**](ListAttacks.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Retrieve a list of all attacks the user has access to |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_attack**
> SimpleAttack get_attack(uuid)

Retrieve an attack by id

Retrieve an attack by id

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.simple_attack import SimpleAttack
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Retrieve an attack by id
        api_response = await api_instance.get_attack(uuid)
        print("The response of AttacksApi->get_attack:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->get_attack: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

[**SimpleAttack**](SimpleAttack.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns the attack |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_bruteforce_subdomains_results**
> BruteforceSubdomainsResultsPage get_bruteforce_subdomains_results(uuid, limit, offset)

Retrieve a bruteforce subdomains' results by the attack's id

Retrieve a bruteforce subdomains' results by the attack's id

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.bruteforce_subdomains_results_page import BruteforceSubdomainsResultsPage
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    uuid = 'uuid_example' # str | 
    limit = 50 # int | Number of items to retrieve
    offset = 0 # int | Position in the whole list to start retrieving from

    try:
        # Retrieve a bruteforce subdomains' results by the attack's id
        api_response = await api_instance.get_bruteforce_subdomains_results(uuid, limit, offset)
        print("The response of AttacksApi->get_bruteforce_subdomains_results:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->get_bruteforce_subdomains_results: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **limit** | **int**| Number of items to retrieve | 
 **offset** | **int**| Position in the whole list to start retrieving from | 

### Return type

[**BruteforceSubdomainsResultsPage**](BruteforceSubdomainsResultsPage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns attack&#39;s results |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_dns_resolution_results**
> DnsResolutionResultsPage get_dns_resolution_results(uuid, limit, offset)

Retrieve a dns resolution's results by the attack's id

Retrieve a dns resolution's results by the attack's id

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.dns_resolution_results_page import DnsResolutionResultsPage
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    uuid = 'uuid_example' # str | 
    limit = 50 # int | Number of items to retrieve
    offset = 0 # int | Position in the whole list to start retrieving from

    try:
        # Retrieve a dns resolution's results by the attack's id
        api_response = await api_instance.get_dns_resolution_results(uuid, limit, offset)
        print("The response of AttacksApi->get_dns_resolution_results:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->get_dns_resolution_results: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **limit** | **int**| Number of items to retrieve | 
 **offset** | **int**| Position in the whole list to start retrieving from | 

### Return type

[**DnsResolutionResultsPage**](DnsResolutionResultsPage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns attack&#39;s results |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_host_alive_results**
> HostAliveResultsPage get_host_alive_results(uuid, limit, offset)

Retrieve a host alive's results by the attack's id

Retrieve a host alive's results by the attack's id

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.host_alive_results_page import HostAliveResultsPage
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    uuid = 'uuid_example' # str | 
    limit = 50 # int | Number of items to retrieve
    offset = 0 # int | Position in the whole list to start retrieving from

    try:
        # Retrieve a host alive's results by the attack's id
        api_response = await api_instance.get_host_alive_results(uuid, limit, offset)
        print("The response of AttacksApi->get_host_alive_results:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->get_host_alive_results: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **limit** | **int**| Number of items to retrieve | 
 **offset** | **int**| Position in the whole list to start retrieving from | 

### Return type

[**HostAliveResultsPage**](HostAliveResultsPage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns attack&#39;s results |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_query_certificate_transparency_results**
> QueryCertificateTransparencyResultsPage get_query_certificate_transparency_results(uuid, limit, offset)

Retrieve a query certificate transparency's results by the attack's id

Retrieve a query certificate transparency's results by the attack's id

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.query_certificate_transparency_results_page import QueryCertificateTransparencyResultsPage
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    uuid = 'uuid_example' # str | 
    limit = 50 # int | Number of items to retrieve
    offset = 0 # int | Position in the whole list to start retrieving from

    try:
        # Retrieve a query certificate transparency's results by the attack's id
        api_response = await api_instance.get_query_certificate_transparency_results(uuid, limit, offset)
        print("The response of AttacksApi->get_query_certificate_transparency_results:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->get_query_certificate_transparency_results: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **limit** | **int**| Number of items to retrieve | 
 **offset** | **int**| Position in the whole list to start retrieving from | 

### Return type

[**QueryCertificateTransparencyResultsPage**](QueryCertificateTransparencyResultsPage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns attack&#39;s results |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_query_unhashed_results**
> QueryUnhashedResultsPage get_query_unhashed_results(uuid, limit, offset)

Retrieve a query dehashed's results by the attack's id

Retrieve a query dehashed's results by the attack's id

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.query_unhashed_results_page import QueryUnhashedResultsPage
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    uuid = 'uuid_example' # str | 
    limit = 50 # int | Number of items to retrieve
    offset = 0 # int | Position in the whole list to start retrieving from

    try:
        # Retrieve a query dehashed's results by the attack's id
        api_response = await api_instance.get_query_unhashed_results(uuid, limit, offset)
        print("The response of AttacksApi->get_query_unhashed_results:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->get_query_unhashed_results: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **limit** | **int**| Number of items to retrieve | 
 **offset** | **int**| Position in the whole list to start retrieving from | 

### Return type

[**QueryUnhashedResultsPage**](QueryUnhashedResultsPage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns attack&#39;s results |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_service_detection_results**
> ServiceDetectionResultsPage get_service_detection_results(uuid, limit, offset)

Retrieve a detect service's results by the attack's id

Retrieve a detect service's results by the attack's id

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.service_detection_results_page import ServiceDetectionResultsPage
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    uuid = 'uuid_example' # str | 
    limit = 50 # int | Number of items to retrieve
    offset = 0 # int | Position in the whole list to start retrieving from

    try:
        # Retrieve a detect service's results by the attack's id
        api_response = await api_instance.get_service_detection_results(uuid, limit, offset)
        print("The response of AttacksApi->get_service_detection_results:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->get_service_detection_results: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **limit** | **int**| Number of items to retrieve | 
 **offset** | **int**| Position in the whole list to start retrieving from | 

### Return type

[**ServiceDetectionResultsPage**](ServiceDetectionResultsPage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns attack&#39;s results |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_tcp_port_scan_results**
> TcpPortScanResultsPage get_tcp_port_scan_results(uuid, limit, offset)

Retrieve a tcp port scan's results by the attack's id

Retrieve a tcp port scan's results by the attack's id

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.tcp_port_scan_results_page import TcpPortScanResultsPage
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    uuid = 'uuid_example' # str | 
    limit = 50 # int | Number of items to retrieve
    offset = 0 # int | Position in the whole list to start retrieving from

    try:
        # Retrieve a tcp port scan's results by the attack's id
        api_response = await api_instance.get_tcp_port_scan_results(uuid, limit, offset)
        print("The response of AttacksApi->get_tcp_port_scan_results:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->get_tcp_port_scan_results: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **limit** | **int**| Number of items to retrieve | 
 **offset** | **int**| Position in the whole list to start retrieving from | 

### Return type

[**TcpPortScanResultsPage**](TcpPortScanResultsPage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Returns attack&#39;s results |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_workspace_attacks**
> ListAttacks get_workspace_attacks(uuid)

Query all attacks of a workspace

Query all attacks of a workspace

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.list_attacks import ListAttacks
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    uuid = 'uuid_example' # str | 

    try:
        # Query all attacks of a workspace
        api_response = await api_instance.get_workspace_attacks(uuid)
        print("The response of AttacksApi->get_workspace_attacks:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->get_workspace_attacks: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 

### Return type

[**ListAttacks**](ListAttacks.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Retrieve a list of all attacks of a workspace |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **hosts_alive_check**
> UuidResponse hosts_alive_check(hosts_alive_request)

Check if hosts are reachable

Check if hosts are reachable  Just an ICMP scan for now to see which targets respond.  All intervals are interpreted in milliseconds. E.g. a `timeout` of 3000 means 3 seconds.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.hosts_alive_request import HostsAliveRequest
from kraken_sdk.models.uuid_response import UuidResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    hosts_alive_request = kraken_sdk.HostsAliveRequest() # HostsAliveRequest | 

    try:
        # Check if hosts are reachable
        api_response = await api_instance.hosts_alive_check(hosts_alive_request)
        print("The response of AttacksApi->hosts_alive_check:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->hosts_alive_check: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **hosts_alive_request** | [**HostsAliveRequest**](HostsAliveRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**202** | Attack scheduled |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **query_certificate_transparency**
> UuidResponse query_certificate_transparency(query_certificate_transparency_request)

Query a certificate transparency log collector.

Query a certificate transparency log collector.  For further information, see [the explanation](https://certificate.transparency.dev/).  Certificate transparency can be used to find subdomains or related domains.  `retry_interval` is specified in milliseconds.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.query_certificate_transparency_request import QueryCertificateTransparencyRequest
from kraken_sdk.models.uuid_response import UuidResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    query_certificate_transparency_request = kraken_sdk.QueryCertificateTransparencyRequest() # QueryCertificateTransparencyRequest | 

    try:
        # Query a certificate transparency log collector.
        api_response = await api_instance.query_certificate_transparency(query_certificate_transparency_request)
        print("The response of AttacksApi->query_certificate_transparency:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->query_certificate_transparency: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **query_certificate_transparency_request** | [**QueryCertificateTransparencyRequest**](QueryCertificateTransparencyRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**202** | Attack scheduled |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **query_dehashed**
> UuidResponse query_dehashed(query_dehashed_request)

Query the [dehashed](https://dehashed.com/) API.

Query the [dehashed](https://dehashed.com/) API. It provides email, password, credit cards and other types of information from leak-databases.  Note that you are only able to query the API if you have bought access and have a running subscription saved in kraken.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.query_dehashed_request import QueryDehashedRequest
from kraken_sdk.models.uuid_response import UuidResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    query_dehashed_request = kraken_sdk.QueryDehashedRequest() # QueryDehashedRequest | 

    try:
        # Query the [dehashed](https://dehashed.com/) API.
        api_response = await api_instance.query_dehashed(query_dehashed_request)
        print("The response of AttacksApi->query_dehashed:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->query_dehashed: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **query_dehashed_request** | [**QueryDehashedRequest**](QueryDehashedRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**202** | Attack scheduled |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **scan_tcp_ports**
> UuidResponse scan_tcp_ports(scan_tcp_ports_request)

Start a tcp port scan

Start a tcp port scan  `exclude` accepts a list of ip networks in CIDR notation.  All intervals are interpreted in milliseconds. E.g. a `timeout` of 3000 means 3 seconds.  Set `max_retries` to 0 if you don't want to try a port more than 1 time.

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.scan_tcp_ports_request import ScanTcpPortsRequest
from kraken_sdk.models.uuid_response import UuidResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    scan_tcp_ports_request = kraken_sdk.ScanTcpPortsRequest() # ScanTcpPortsRequest | 

    try:
        # Start a tcp port scan
        api_response = await api_instance.scan_tcp_ports(scan_tcp_ports_request)
        print("The response of AttacksApi->scan_tcp_ports:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->scan_tcp_ports: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **scan_tcp_ports_request** | [**ScanTcpPortsRequest**](ScanTcpPortsRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**202** | Attack scheduled |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **service_detection**
> UuidResponse service_detection(service_detection_request)

Perform service detection on a ip and port combination

Perform service detection on a ip and port combination

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.service_detection_request import ServiceDetectionRequest
from kraken_sdk.models.uuid_response import UuidResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.AttacksApi(api_client)
    service_detection_request = kraken_sdk.ServiceDetectionRequest() # ServiceDetectionRequest | 

    try:
        # Perform service detection on a ip and port combination
        api_response = await api_instance.service_detection(service_detection_request)
        print("The response of AttacksApi->service_detection:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling AttacksApi->service_detection: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **service_detection_request** | [**ServiceDetectionRequest**](ServiceDetectionRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**202** | Attack scheduled |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

