import { normalizeAmount, trimZeros, convertToReal } from '../src/services/near-nep21-util'
import { assert } from "chai";

describe("Normalize Accounts ", () => {
    
    it("should trim zeros ", () => {
        var res = trimZeros("000.12");
        assert.equal(res, ".12", 'mismatch');

        res = trimZeros("0.32");
        assert.equal(res, ".32", 'mismatch');

        res = trimZeros("5.32");
        assert.equal(res, "5.32", 'mismatch');

        res = trimZeros("0.32000");
        assert.equal(res, ".32", 'mismatch');

        res = trimZeros("1000");
        assert.equal(res, "1000", 'mismatch');

        res = trimZeros("0020");
        assert.equal(res, "20", 'mismatch');
    });

    it("normalize accounts ", () => {
        var res = normalizeAmount(".12");
        assert.equal(res, "120000000000000000000000", 'mismatch');

        var res = normalizeAmount("42");
        assert.equal(res, "42000000000000000000000000", 'mismatch');
    });
});

describe("Convert to real number", () => {

    it("Division ",  () => {
        var res = convertToReal("1000000000000000000000000");
        assert.equal(res, "1", 'mismatch');
        
        var res = convertToReal("10000000000000000000000");
        assert.equal(res, "0.01", 'mismatch');

        var res = convertToReal("1234567891234567891234567");
        assert.equal(res, "1.234567891234567891234567", 'mismatch');

        var res = convertToReal("1234567891234567894567");
        assert.equal(res, "0.001234567891234567894567", 'mismatch');
    });
});