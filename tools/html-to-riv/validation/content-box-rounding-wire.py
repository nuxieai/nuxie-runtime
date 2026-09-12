"""Read the compiler's ordinary field-table encoding; no rewrite or import hooks."""
import struct

def records(data):
    assert data[:7] == b'RIVE\x07\x03\x00'
    pos = 7
    def uint():
        nonlocal pos
        value = shift = 0
        while True:
            b = data[pos]; pos += 1; value |= (b & 127) << shift
            if b < 128: return value
            shift += 7
            assert shift < 35
    keys = []
    while key := uint(): keys.append(key)
    fields = {}
    for offset in range(0, len(keys), 4):
        packed = struct.unpack_from('<I',data,pos)[0]; pos += 4
        for i,key in enumerate(keys[offset:offset+4]): fields[key] = (packed >> (i*2)) & 3
    result = []
    while pos < len(data):
        kind = uint(); properties = {}
        while key := uint():
            field = fields[key]; start = pos
            if field == 0: value = uint()
            elif field == 1:
                length = uint(); value = data[pos:pos+length].hex(); pos += length
            else:
                value = struct.unpack_from('<f' if field==2 else '<I',data,pos)[0]; pos += 4
            properties[key] = dict(value=value, offset=start, field=field)
        result.append(dict(kind=kind,properties=properties))
    assert pos == len(data)
    return result
