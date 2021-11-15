require ["fileinto", "envelope"];

# Whitelist @domain.com
if envelope :domain :is "to" "domain.com" {
    # Generic Filters
    if envelope :localpart :matches "to" ["newsletter.secret","newsletter.secret.*"] {
        fileinto "Newsletter";
    } elsif envelope :localpart :matches "to" ["newsletter.business.secret","newsletter.business.secret.*"] {
        fileinto "Newsletter/Business";
    } elsif envelope :localpart :matches "to" ["newsletter.software.secret","newsletter.software.secret.*"] {
        fileinto "Newsletter/Software";
    } elsif envelope :localpart :matches "to" ["utilities.secret","utilities.secret.*"] {
        fileinto "Utilities";
    } elsif envelope :localpart :matches "to" ["utilities.genericonly.secret","utilities.genericonly.secret.*","genericonly.secret","genericonly.secret.*"] {
        fileinto "Utilities/GenericOnly";
    } elsif envelope :localpart :matches "to" ["utilities.grocery.secret","utilities.grocery.secret.*","grocery.secret","grocery.secret.*"] {
        fileinto "Utilities/Grocery";
        if header :contains "subject" ["keyword"] {
            fileinto "label";
        }
        if header :contains "subject" ["keyword2"] {
            fileinto "label2";
        }
    } else {
        fileinto "Unknown";
    }
    # Custom Filters
    if envelope :localpart :matches "to" ["wallstreet"] {
        fileinto "Newsletter/Business";
    } elsif envelope :localpart :matches "to" ["google","facebook"] {
        fileinto "Newsletter/Software";
    } elsif envelope :localpart :matches "to" ["market"] {
        fileinto "Utilities/Grocery";
        if header :contains "subject" ["keyword"] {
            fileinto "label";
        }
        if header :contains "subject" ["keyword2"] {
            fileinto "label2";
        }
    }
}