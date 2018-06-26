//use indexmap::map::Entry;
use juniper::parser::Spanning;
use juniper::*;

pub fn resolve_selection_set_into<T, CtxT>(
    instance: &T,
    info: &T::TypeInfo,
    selection_set: &[Selection],
    executor: &Executor<CtxT>,
    result: &mut Object,
) -> bool
where
    T: GraphQLType<Context = CtxT>,
{
    let meta_type = executor
        .schema()
        .concrete_type_by_name(T::name(info).expect("Resolving named type's selection set"))
        .expect("Type not found in schema");
    //    let look_ahead = executor.look_ahead();
    for selection in selection_set {
        match *selection {
            Selection::Field(Spanning {
                ref item,
                ref start,
                ..
            }) => {
                let response_name = &item.alias.as_ref().unwrap_or(&item.name).item;

                if item.name.item == "__typename" {
                    result.add_field(
                        (*response_name).to_owned(),
                        Value::string(instance.concrete_type_name(executor.context(), info)),
                    );
                    continue;
                }

                let meta_field = meta_type.field_by_name(item.name.item).unwrap_or_else(|| {
                    panic!(format!(
                        "Field {} not found on type {:?}",
                        item.name.item,
                        meta_type.name()
                    ))
                });

                //                let exec_vars = executor.variables();

                let sub_exec = executor.field_sub_executor(
                    response_name,
                    item.name.item,
                    *start,
                    item.selection_set.as_ref().map(|v| v as &[_]),
                );

                let field_result = instance.resolve_field(
                    info,
                    item.name.item,
                    &Arguments::new(None, &meta_field.arguments),
                    &sub_exec,
                );

                match field_result {
                    Ok(Value::Null) if meta_field.field_type.is_non_null() => return false,
                    Ok(v) => {
                        result.add_field((*response_name).to_owned(), v);
                    }
                    Err(e) => {
                        sub_exec.push_error_at(e, start.clone());

                        if meta_field.field_type.is_non_null() {
                            return false;
                        }

                        result.add_field((*response_name).to_owned(), Value::null());
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
    true
}

// pub fn resolve_selection_set_into1<T, CtxT>(
//     instance: &T,
//     info: &T::TypeInfo,
//     selection_set: &[Selection],
//     executor: &Executor<CtxT>,
//     result: &mut IndexMap<String, Value>,
// ) -> bool
// where
//     T: GraphQLType<Context = CtxT>,
// {
//     let meta_type = executor
//         .schema()
//         .concrete_type_by_name(T::name(info).expect("Resolving named type's selection set"))
//         .expect("Type not found in schema");

//     for selection in selection_set {
//         match *selection {
//             Selection::Field(Spanning {
//                 item: ref f,
//                 start: ref start_pos,
//                 ..
//             }) => {
//                 // TODO: fix this
//                 // if is_excluded(&f.directives, executor.variables()) {
//                 //     continue;
//                 // }

//                 let response_name = &f.alias.as_ref().unwrap_or(&f.name).item;

//                 if f.name.item == "__typename" {
//                     result.insert(
//                         (*response_name).to_owned(),
//                         Value::string(instance.concrete_type_name(executor.context(), info)),
//                     );
//                     continue;
//                 }

//                 let meta_field = meta_type.field_by_name(f.name.item).unwrap_or_else(|| {
//                     panic!(format!(
//                         "Field {} not found on type {:?}",
//                         f.name.item,
//                         meta_type.name()
//                     ))
//                 });

//                 //                let exec_vars = executor.variables();

//                 let sub_exec = executor.field_sub_executor(
//                     response_name,
//                     f.name.item,
//                     start_pos.clone(),
//                     f.selection_set.as_ref().map(|v| &v[..]),
//                 );

//                 let field_result = instance.resolve_field(
//                     info,
//                     f.name.item,
//                     &Arguments::new(None, &meta_field.arguments),
//                     &sub_exec,
//                 );

//                 match field_result {
//                     Ok(Value::Null) if meta_field.field_type.is_non_null() => return false,
//                     Ok(v) => merge_key_into(result, response_name, v),
//                     Err(e) => {
//                         sub_exec.push_error_at(e, start_pos.clone());

//                         if meta_field.field_type.is_non_null() {
//                             return false;
//                         }

//                         result.insert((*response_name).to_owned(), Value::null());
//                     }
//                 }
//             }
//             Selection::FragmentSpread(Spanning {
//                 item: ref spread, ..
//             }) => {
//                 // TODO: fix this
//                 // if is_excluded(&spread.directives, executor.variables()) {
//                 //     continue;
//                 // }

//                 let fragment = &executor
//                     .fragment_by_name(spread.name.item)
//                     .expect("Fragment could not be found");

//                 if !resolve_selection_set_into(
//                     instance,
//                     info,
//                     &fragment.selection_set[..],
//                     executor,
//                     result,
//                 ) {
//                     return false;
//                 }
//             }
//             Selection::InlineFragment(Spanning {
//                 item: ref fragment,
//                 start: ref start_pos,
//                 ..
//             }) => {
//                 // TODO: fix this
//                 // if is_excluded(&fragment.directives, executor.variables()) {
//                 //     continue;
//                 // }

//                 let sub_exec = executor.type_sub_executor(
//                     fragment.type_condition.as_ref().map(|c| c.item),
//                     Some(&fragment.selection_set[..]),
//                 );

//                 if let Some(ref type_condition) = fragment.type_condition {
//                     let sub_result = instance.resolve_into_type(
//                         info,
//                         type_condition.item,
//                         Some(&fragment.selection_set[..]),
//                         &sub_exec,
//                     );

//                     if let Ok(Value::Object(mut hash_map)) = sub_result {
//                         for (k, v) in hash_map.drain(..) {
//                             merge_key_into(result, &k, v);
//                         }
//                     } else if let Err(e) = sub_result {
//                         sub_exec.push_error_at(e, start_pos.clone());
//                     }
//                 } else if !resolve_selection_set_into(
//                     instance,
//                     info,
//                     &fragment.selection_set[..],
//                     &sub_exec,
//                     result,
//                 ) {
//                     return false;
//                 }
//             }
//         }
//     }

//     true
// }

// fn is_excluded(directives: &Option<Vec<Spanning<Directive>>>, vars: &Variables) -> bool {
//     if let Some(ref directives) = *directives {
//         for &Spanning {
//             item: ref directive,
//             ..
//         } in directives
//         {
//             let condition: bool = directive
//                 .arguments
//                 .iter()
//                 .flat_map(|m| m.item.get("if"))
//                 .flat_map(|v| v.item.clone().into_const(vars).convert())
//                 .next()
//                 .unwrap();

//             if (directive.name.item == "skip" && condition)
//                 || (directive.name.item == "include" && !condition)
//             {
//                 return true;
//             }
//         }
//     }
//     false
// }

// fn merge_key_into(result: &mut IndexMap<String, Value>, response_name: &str, value: Value) {
//     match result.entry(response_name.to_owned()) {
//         Entry::Occupied(mut e) => match *e.get_mut() {
//             Value::Object(ref mut dest_obj) => {
//                 if let Value::Object(src_obj) = value {
//                     merge_maps(dest_obj, src_obj);
//                 }
//             }
//             Value::List(ref mut dest_list) => {
//                 if let Value::List(src_list) = value {
//                     dest_list
//                         .iter_mut()
//                         .zip(src_list.into_iter())
//                         .for_each(|(d, s)| {
//                             if let Value::Object(ref mut d_obj) = *d {
//                                 if let Value::Object(s_obj) = s {
//                                     merge_maps(d_obj, s_obj);
//                                 }
//                             }
//                         });
//                 }
//             }
//             _ => {}
//         },
//         Entry::Vacant(e) => {
//             e.insert(value);
//         }
//     }
// }

// fn merge_maps(dest: &mut IndexMap<String, Value>, src: IndexMap<String, Value>) {
//     for (key, value) in src {
//         if dest.contains_key(&key) {
//             merge_key_into(dest, &key, value);
//         } else {
//             dest.insert(key, value);
//         }
//     }
// }
