function _instanceof(left, right) {
    if (right != null && typeof Symbol !== "undefined" && right[Symbol.hasInstance]) {
        return right[Symbol.hasInstance](left);
    } else {
        return left instanceof right;
    }
}
//@target: ES6
_instanceof(Symbol(), Symbol);
_instanceof(Symbol, Symbol());
_instanceof(Symbol() || {}, Object); // This one should be okay, it's a valid way of distinguishing types
_instanceof(Symbol, Symbol() || {});
