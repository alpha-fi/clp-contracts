export function get (...paramNames) {
  const params = new URLSearchParams(window.location.search)

  if (paramNames.length === 1) {
    return params.get(paramNames[0])
  }

  return paramNames.reduce(
    (obj, paramName) => ({ ...obj, [paramName]: params.get(paramName) }),
    {}
  )
}

export function set (newParams) {
  const params = new URLSearchParams(window.location.search)
  for (const param in newParams) {
    params.set(param, newParams[param])
  }
  window.history.replaceState({}, '', `${location.pathname}?${params}`)
}

export function clear (...paramNames) {
  if (paramNames.length === 0) {
    window.history.replaceState({}, '', `${location.pathname}`)
  } else {
    const params = new URLSearchParams(window.location.search)
    paramNames.forEach(p => params.delete(p))
    window.history.replaceState({}, '', `${location.pathname}?${params}`)
  }
}
