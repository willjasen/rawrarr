"""ElementTree helpers that reject document type and entity declarations."""

import xml.etree.ElementTree as ET
from xml.parsers import expat


def fromstring(data):
    """Parse XML only after Expat confirms it contains no DTD or entities."""
    parser = expat.ParserCreate()

    def reject_declaration(*_args):
        raise ValueError("DTD and entity declarations are not allowed")

    parser.StartDoctypeDeclHandler = reject_declaration
    parser.EntityDeclHandler = reject_declaration
    parser.ExternalEntityRefHandler = reject_declaration
    try:
        parser.Parse(data, True)
    except expat.ExpatError as error:
        raise ValueError("invalid XML configuration") from error
    return ET.fromstring(data)
