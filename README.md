# sieve-generator

This is a CLI used to generate `.sieve` email filter allowlist for different folders based on the receiver email address, configured with a `JSON` file. I have a catch-all email domain and use it automates the allowlist creating process whenever I use a new email. (e.g. `something@domain.com` filtered to `something`
folder)

### Usage

Let say we have `JSON` config file like this

```
{
    "domain.com": {
        "options": { // global options
            // if every folder should be a sub-folder of a domain named folder
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
                // if title or sender contains keyword then go to label
                // by default every labeled mail will not be marked read
                // even if silent is set
                "labels": {
                    "label": "keyword",
                    "label2": ["keyword2"]
                },
                "options": {
                    // the generic filter here will be grocery.secret and
                    // not utilities.grocery.secret
                    "orphan": true,
                    // mark all email of this folder as read
                    "silent": true,
                    // not generate generic filter for this folder
                    "generic": false
                }
            }
        }
    }
}
```

A `.sieve` filter allowlist will be generated in which

- It applies to all email sent to `*@domain.com`
- `wallstreet@domain.com` will be filtered to `Business` folder of
  `Newsletter` parent folder and so on.
- The filter for a parent folder with children (e.g. `Utilities`) is set using
  `self` keyword.
- There are short form filter (e.g. `Business` folder) and full form (e.g.
  `Grocery`) that allows to specify options and a label rule.
- There is a generic filter generated for each in the folder tree, e.g.
  `newsletter.business.*@domain.com` is filtered to `Business` folder.
- Every mails that are not allowlisted will be put in an `Unknown` folder by
    default.
- `Unknown` is a special folder, everything go there will be silent, even if explicitly configured.
