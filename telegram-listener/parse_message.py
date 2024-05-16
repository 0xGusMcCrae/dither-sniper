import re
import json

# example message format
"""
Address: 6X7XXRb2bfoEogRXHBEDxPeyKVMPSNaQd8uCTU3biTqQ

Name: hillury clinton (CLINTON)

📊Ratings📊
Memeability: 10.0/10
AI Degen:
☠️- Death 22.42%
🔴 - Red 45.33%
🟡 - Yellow 23.12%
🟢 - Green 9.13%
Name Originality: 8.6/10
Description Originality: 1/10

🏦Financials🏦
Price: $0.000175
Liquidity: $26.81k
Latest Marketcap: $175.37k
Transactions: 68
5m Price Change: 138.56%
Volume: $205.89M

🔒Security🔒
Top 20 Holders: 50.3%
Total Holders: 340
Token Supply: 999.78M

Seer v1.54
Disclaimer: Seer is an experimental research tool. Seer is not intended as financial advice and any actions taken after consuming seer content is of the sole volition of >
"""

def parse_message(message):
    data = {
        'Address': None,
        'Name': None,
        'Ratings': {},
        'Financials': {},
        'Security': {}
    }
    try:
        data['Address'] = re.search(r'Address: (\S+)', message).group(1)
    except AttributeError:
        data['Address'] = "Not found"
        return None
        
    try:
        data['Name'] = re.search(r'Name: ([\w\s]+) \((\w+)\)', message).group(1)
    except AttributeError:
        data['Name'] = "Not found"
        
    ratings = data['Ratings']
    try:
        ratings['Memeability'] = re.search(r'Memeability: ([\d\.]+)/10', message).group(1)
    except AttributeError:
        ratings['Memeability'] = "Not found"

    try:
        ratings['Death'] = re.search(r'☠️- Death ([\d\.]+)%', message).group(1)
    except AttributeError:
        ratings['Death'] = "Not found"
    
    try:
        ratings['Red'] = re.search(r'🔴 - Red ([\d\.]+)%', message).group(1)
    except AttributeError:
        ratings['Red'] = "Not found"
    
    try:
        ratings['Yellow'] = re.search(r'🟡 - Yellow ([\d\.]+)%', message).group(1)
    except AttributeError:
        ratings['Yellow'] = "Not found"
    
    try:
        ratings['Green'] = re.search(r'🟢 - Green ([\d\.]+)%', message).group(1)
    except AttributeError:
        ratings['Green'] = "Not found"
    
    try:
        ratings['Name Originality'] = re.search(r'Name Originality: ([\d\.]+)/10', message).group(1)
    except AttributeError:
        ratings['Name Originality'] = "Not found"
    
    try:
        ratings['Description Originality'] = re.search(r'Description Originality: ([\d\.]+)/10', message).group(1)
    except AttributeError:
        ratings['Description Originality'] = "Not found"

    financials = data['Financials']
    try:
        financials['Price'] = re.search(r'Price: \$(\S+)', message).group(1)
    except AttributeError:
        financials['Price'] = "Not found"

    try:
        financials['Liquidity'] = re.search(r'Liquidity: \$(\S+)', message).group(1)
    except AttributeError:
        financials['Liquidity'] = "Not found"

    try:
        financials['Latest Marketcap'] = re.search(r'Latest Marketcap: \$(\S+)', message).group(1)
    except AttributeError:
        financials['Latest Marketcap'] = "Not found"

    try:
        financials['Transactions'] = re.search(r'Transactions: (\d+)', message).group(1)
    except AttributeError:
        financials['Transactions'] = "Not found"

    try:
        financials['5m Price Change'] = re.search(r'5m Price Change: ([\d\.]+)%', message).group(1)
    except AttributeError:
        financials['5m Price Change'] = "Not found"

    try:
        financials['Volume'] = re.search(r'Volume: \$(\S+)', message).group(1)
    except AttributeError:
        financials['Volume'] = "Not found"

    security = data['Security']
    try:
        security['Top 20 Holders'] = re.search(r'Top 20 Holders: ([\d\.]+)%', message).group(1)
    except AttributeError:
        security['Top 20 Holders'] = "Not found"

    try:
        security['Total Holders'] = re.search(r'Total Holders: (\d+)', message).group(1)
    except AttributeError:
        security['Total Holders'] = "Not found"

    try:
        security['Token Supply'] = re.search(r'Token Supply: (\S+)', message).group(1)
    except AttributeError:
        security['Token Supply'] = "Not found"

    return json.dumps(data)
