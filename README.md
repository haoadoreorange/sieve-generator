# sieve-generator

This is used to generate `.sieve` email filter whitelist for different folders
based on the receiver email address from a config `JSON` file. I have a
catch-all email domain and use it automates the whitelist creating process
whenever I use a new email (e.g `something@domain.com` filtered to `something`
folder)

### Usage

Let say we have `JSON` config file like this

```
{
    "domain.com": {
        "options": { // global options
            // if every folder should be a sub-folder of a domain named folder
            "domain-as-first-folder": false,
            // list of possible secrets
            "secrets": ["secret"]
        },
        "Newsletter": {
            "Software": ["google", "facebook"],
            "Business": "wallstreet"
        },
        "Utilities": {
            // the generic filter here will be grocery.secret and
            // not utilities.grocery.secret
            "fakeroot": true,
            "self": ["electricity"],
            "Grocery": {
                // not generate generic filter for this folder
                "generic": false,
                // mark all email of this folder as read
                "silent": true,
                "localparts": "market",
                // if title contains keyword then go to label
                // by default every labeled mail will not be marked read
                // even if the setting is set
                "labels": {
                    "label": "keyword",
                    "label2": ["keyword2"]
                }
            }
        }
    }
}
```

A `.sieve` filter whitelist will be generated in which

-   It applies to all email sent to `*@domain.com`
-   `wallstreet@domain.com` will be filtered to `Business` folder of
    `Newsletter` parent folder and so on.
-   The filter for a parent folder with children (e.g `Utilities`) is set using
    `self` keyword.
-   There are short form filter (e.g `Business` folder) and full form (e.g
    `Grocery`) that allows to specify options and a label rule.
-   There is a generic filter generated for each in the folder tree, e.g
    `newsletter.business.secret@domain.com` is filtered to `Business` folder,
    the secret make it harder for spam mails.
-   Every mails that are not whitelisted will be put in an `Unknown` folder by
    default.
