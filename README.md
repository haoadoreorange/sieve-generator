# sieve-generator

This is a CLI used to generate `.sieve` email filter allowlist for different folders based on the receiver email address, configured with a `JSON` file. I have a catch-all email domain and use it automates the allowlist creating process whenever I use a new email. (e.g. `something@domain.com` filtered to `something` folder)

### Installation

`cargo install sieve-generator` or download the binary, only tested on Linux.

### Usage

Let say we have `JSON` config file like this

```
{
    "domain.com": {
        "options": { // Global options
            // If every folder should be a sub-folder of a domain named folder
            "domain-as-first-folder": false,
        },
        "Newsletter": {
            "Software": ["google", "facebook"],
            "Business": "wallstreet"
        },
        "Utilities": {
            "self": ["electricity"],
            "Grocery": {
                "localparts": "market",
                // If title or sender contains keyword then go to label
                // By default every labeled mail will not be marked read
                // even if mark-as-read is set
                "labels": {
                    "label": "keyword",
                    "label2": ["keyword2"]
                },
                "options": {
                    // The generic filter here will be utilities.grocery.secret
                    // not grocery.secret
                    "fullpath": true,
                    // Mark all email of this folder as read
                    "mark-as-read": true,
                    // Not generate generic filter for this folder
                    "generic": false
                }
            }
        }
    }
}
```

A `.sieve` filter allowlist will be generated in which

- It applies to all email sent to `*@domain.com`
- `wallstreet@domain.com` will be filtered to `Business` folder of `Newsletter` parent folder and so on.
- The filter for a parent folder with children (e.g. `Utilities`) is set using `self` keyword.
- There are short form filter (e.g. `Business` folder) and full form (e.g.  `Grocery`) that allows to specify options and a label rule.
- There is a generic filter generated for each in the folder tree, e.g.  `newsletter.business.*@domain.com` (fullpath option == `true`) is filtered to `Business` folder.
- Every mails that are not allowlisted will be put in an `Unknown` folder by default.
- `Unknown` is a special folder, everything go there will be silent, even if explicitly configured.

The `JSON` above will produce

```
# @domain.com
if envelope :domain :is "to" "domain.com" {
    # Custom filters
    if envelope :localpart :matches "to" ["market"] {
        if header :contains ["from","subject"] ["keyword","keyword2"] {
            if header :contains ["from","subject"] ["keyword"] {
                fileinto "label";
            }
            if header :contains ["from","subject"] ["keyword2"] {
                fileinto "label2";
            }
        } else {
            addflag "\\Seen";
            fileinto "unread";
        }
        fileinto "Utilities";
        fileinto "Utilities/Grocery";
    } elsif envelope :localpart :matches "to" ["electricity"] {
        fileinto "Utilities";
    } elsif envelope :localpart :matches "to" ["google","facebook"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Software";
    } elsif envelope :localpart :matches "to" ["wallstreet"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Business";
    }
    # Generic filters
    elsif envelope :localpart :matches "to" ["utilities","utilities.*"] {
        fileinto "Utilities";
    } elsif envelope :localpart :matches "to" ["software","software.*"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Software";
    } elsif envelope :localpart :matches "to" ["business","business.*"] {
        fileinto "Newsletter";
        fileinto "Newsletter/Business";
    } elsif envelope :localpart :matches "to" ["newsletter","newsletter.*"] {
        fileinto "Newsletter";
    } else {
        addflag "\\Seen";
        fileinto "Unknown";
    }
}
```
