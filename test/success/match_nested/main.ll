@.str_0 = private unnamed_addr constant [1 x i8] c"4", align 1
@.str_1 = private unnamed_addr constant [3 x i8] c"foo", align 1
%struct.Bool_Bool = type { i32, [0 x i8] }

%struct.Bool_Bool_False = type { i32, %struct.siko_Tuple_ }

%struct.Bool_Bool_True = type { i32, %struct.siko_Tuple_ }

%struct.Int_Int = type { i64 }

%struct.Main_Foo = type { i32, [4 x i8] }

%struct.Main_Foo_Bar = type { i32, %struct.siko_Tuple_Bool_Bool }

%struct.String_String = type { ptr, i64 }

%struct.siko_Tuple_ = type {  }

%struct.siko_Tuple_Bool_Bool = type { %struct.Bool_Bool }

%struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo = type { %struct.Bool_Bool, %struct.Int_Int, %struct.String_String, %struct.Main_Foo }

define private void @siko_Tuple_(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_, align 4
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 0, i1 false)
   ret void
}

define private void @siko_Tuple_Bool_Bool(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Bool_Bool, align 4
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo(ptr noundef %f0, ptr noundef %f1, ptr noundef %f2, ptr noundef %f3, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, align 8
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %this, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   %field1 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field1, ptr align 8 %f1, i64 8, i1 false)
   %field2 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %this, i32 0, i32 2
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %field2, ptr align 8 %f2, i64 16, i1 false)
   %field3 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %this, i32 0, i32 3
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field3, ptr align 4 %f3, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %fn_result, ptr align 8 %this, i64 40, i1 false)
   ret void
}

define private void @Main_main(ptr noundef %fn_result) {
block0:
   %valueRef_53 = alloca %struct.Int_Int, align 8
   %v_52 = alloca %struct.Int_Int, align 8
   %valueRef_49 = alloca %struct.Int_Int, align 8
   %v_48 = alloca %struct.Int_Int, align 8
   %lit_42 = alloca %struct.Int_Int, align 8
   %valueRef_39 = alloca %struct.Int_Int, align 8
   %v_38 = alloca %struct.Int_Int, align 8
   %valueRef_30 = alloca %struct.Int_Int, align 8
   %v_29 = alloca %struct.Int_Int, align 8
   %valueRef_26 = alloca %struct.Int_Int, align 8
   %v_25 = alloca %struct.Int_Int, align 8
   %eq_35 = alloca %struct.Bool_Bool, align 4
   %implicitRef1 = alloca ptr, align 8
   %lit_34 = alloca %struct.String_String, align 8
   %lit_17 = alloca %struct.Int_Int, align 8
   %lit_14 = alloca %struct.Int_Int, align 8
   %implicitRef0 = alloca ptr, align 8
   %unit_62 = alloca %struct.siko_Tuple_, align 4
   %matchValue_61 = alloca %struct.Int_Int, align 8
   %match_var_7 = alloca %struct.Int_Int, align 8
   %tuple_6 = alloca %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, align 8
   %call_5 = alloca %struct.Main_Foo, align 4
   %call_4 = alloca %struct.Bool_Bool, align 4
   %literal_3 = alloca %struct.String_String, align 8
   %lit_2 = alloca %struct.Int_Int, align 8
   %call_1 = alloca %struct.Bool_Bool, align 4
   call void @Bool_Bool_True(ptr %call_1)
   %tmp_lit_2_1 = getelementptr inbounds %struct.Int_Int, ptr %lit_2, i32 0, i32 0
   store i64 3, ptr %tmp_lit_2_1, align 8
   %tmp_literal_3_2 = getelementptr inbounds %struct.String_String, ptr %literal_3, i32 0, i32 0
   store ptr @.str_0, ptr %tmp_literal_3_2, align 8
   %tmp_literal_3_3 = getelementptr inbounds %struct.String_String, ptr %literal_3, i32 0, i32 1
   store i64 1, ptr %tmp_literal_3_3, align 8
   call void @Bool_Bool_True(ptr %call_4)
   call void @Main_Foo_Bar(ptr %call_4, ptr %call_5)
   call void @siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo(ptr %call_1, ptr %lit_2, ptr %literal_3, ptr %call_5, ptr %tuple_6)
   br label %block2
block1:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %matchValue_61, ptr align 8 %match_var_7, i64 8, i1 false)
   call void @siko_Tuple_(ptr %unit_62)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %unit_62, i64 0, i1 false)
   ret void
block2:
   %tupleField_8 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %tuple_6, i32 0, i32 0
   %tupleField_9 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %tuple_6, i32 0, i32 1
   %tupleField_10 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %tuple_6, i32 0, i32 2
   store ptr %tupleField_10, ptr %implicitRef0, align 8
   %tupleField_11 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool__Int_Int__String_String__Main_Foo, ptr %tuple_6, i32 0, i32 3
   br label %block3
block3:
   %tmp_switch_var_block3_4 = getelementptr inbounds %struct.Bool_Bool, ptr %tupleField_8, i32 0, i32 0
   %tmp_switch_var_block3_5 = load i32, ptr %tmp_switch_var_block3_4, align 4
   switch i32 %tmp_switch_var_block3_5, label %block4 [
i32 1, label %block14
]

block4:
   br label %block5
block5:
   %tmp_switch_var_block5_6 = getelementptr inbounds %struct.Int_Int, ptr %tupleField_9, i32 0, i32 0
   %tmp_switch_var_block5_7 = load i64, ptr %tmp_switch_var_block5_6, align 8
   switch i64 %tmp_switch_var_block5_7, label %block6 [

]

block6:
   br label %block7
block7:
   %tmp_switch_var_block7_8 = getelementptr inbounds %struct.Main_Foo, ptr %tupleField_11, i32 0, i32 0
   %tmp_switch_var_block7_9 = load i32, ptr %tmp_switch_var_block7_8, align 4
   switch i32 %tmp_switch_var_block7_9, label %block8 [

]

block8:
   %tmp_transform_12_10 = bitcast %struct.Main_Foo* %tupleField_11 to %struct.Main_Foo_Bar*
   %transform_12 = getelementptr inbounds %struct.Main_Foo_Bar, ptr %tmp_transform_12_10, i32 0, i32 1
   %tupleField_13 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %transform_12, i32 0, i32 0
   br label %block9
block9:
   %tmp_switch_var_block9_11 = getelementptr inbounds %struct.Bool_Bool, ptr %tupleField_13, i32 0, i32 0
   %tmp_switch_var_block9_12 = load i32, ptr %tmp_switch_var_block9_11, align 4
   switch i32 %tmp_switch_var_block9_12, label %block10 [
i32 1, label %block12
]

block10:
   br label %block11
block11:
   %tmp_lit_14_13 = getelementptr inbounds %struct.Int_Int, ptr %lit_14, i32 0, i32 0
   store i64 3, ptr %tmp_lit_14_13, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_7, ptr align 8 %lit_14, i64 8, i1 false)
   br label %block1
block12:
   br label %block13
block13:
   %tmp_lit_17_14 = getelementptr inbounds %struct.Int_Int, ptr %lit_17, i32 0, i32 0
   store i64 3, ptr %tmp_lit_17_14, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_7, ptr align 8 %lit_17, i64 8, i1 false)
   br label %block1
block14:
   br label %block15
block15:
   %tmp_switch_var_block15_15 = getelementptr inbounds %struct.Int_Int, ptr %tupleField_9, i32 0, i32 0
   %tmp_switch_var_block15_16 = load i64, ptr %tmp_switch_var_block15_15, align 8
   switch i64 %tmp_switch_var_block15_16, label %block31 [
i64 1, label %block16
]

block16:
   %tmp_lit_34_17 = getelementptr inbounds %struct.String_String, ptr %lit_34, i32 0, i32 0
   store ptr @.str_1, ptr %tmp_lit_34_17, align 8
   %tmp_lit_34_18 = getelementptr inbounds %struct.String_String, ptr %lit_34, i32 0, i32 1
   store i64 3, ptr %tmp_lit_34_18, align 8
   store ptr %lit_34, ptr %implicitRef1, align 8
   %tmp_implicitRef0_19 = load ptr, ptr %implicitRef0, align 8
   %tmp_implicitRef1_20 = load ptr, ptr %implicitRef1, align 8
   call void @String_String_eq(ptr %tmp_implicitRef0_19, ptr %tmp_implicitRef1_20, ptr %eq_35)
   %tmp_switch_var_block16_21 = getelementptr inbounds %struct.Bool_Bool, ptr %eq_35, i32 0, i32 0
   %tmp_switch_var_block16_22 = load i32, ptr %tmp_switch_var_block16_21, align 4
   switch i32 %tmp_switch_var_block16_22, label %block17 [
i32 1, label %block24
]

block17:
   %tmp_switch_var_block17_23 = getelementptr inbounds %struct.Main_Foo, ptr %tupleField_11, i32 0, i32 0
   %tmp_switch_var_block17_24 = load i32, ptr %tmp_switch_var_block17_23, align 4
   switch i32 %tmp_switch_var_block17_24, label %block18 [

]

block18:
   %tmp_transform_23_25 = bitcast %struct.Main_Foo* %tupleField_11 to %struct.Main_Foo_Bar*
   %transform_23 = getelementptr inbounds %struct.Main_Foo_Bar, ptr %tmp_transform_23_25, i32 0, i32 1
   %tupleField_24 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %transform_23, i32 0, i32 0
   br label %block19
block19:
   %tmp_switch_var_block19_26 = getelementptr inbounds %struct.Bool_Bool, ptr %tupleField_24, i32 0, i32 0
   %tmp_switch_var_block19_27 = load i32, ptr %tmp_switch_var_block19_26, align 4
   switch i32 %tmp_switch_var_block19_27, label %block20 [
i32 1, label %block22
]

block20:
   br label %block21
block21:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v_25, ptr align 8 %tupleField_9, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_26, ptr align 8 %v_25, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_7, ptr align 8 %valueRef_26, i64 8, i1 false)
   br label %block1
block22:
   br label %block23
block23:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v_29, ptr align 8 %tupleField_9, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_30, ptr align 8 %v_29, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_7, ptr align 8 %valueRef_30, i64 8, i1 false)
   br label %block1
block24:
   %tmp_switch_var_block24_28 = getelementptr inbounds %struct.Main_Foo, ptr %tupleField_11, i32 0, i32 0
   %tmp_switch_var_block24_29 = load i32, ptr %tmp_switch_var_block24_28, align 4
   switch i32 %tmp_switch_var_block24_29, label %block25 [

]

block25:
   %tmp_transform_36_30 = bitcast %struct.Main_Foo* %tupleField_11 to %struct.Main_Foo_Bar*
   %transform_36 = getelementptr inbounds %struct.Main_Foo_Bar, ptr %tmp_transform_36_30, i32 0, i32 1
   %tupleField_37 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %transform_36, i32 0, i32 0
   br label %block26
block26:
   %tmp_switch_var_block26_31 = getelementptr inbounds %struct.Bool_Bool, ptr %tupleField_37, i32 0, i32 0
   %tmp_switch_var_block26_32 = load i32, ptr %tmp_switch_var_block26_31, align 4
   switch i32 %tmp_switch_var_block26_32, label %block27 [
i32 1, label %block29
]

block27:
   br label %block28
block28:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v_38, ptr align 8 %tupleField_9, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_39, ptr align 8 %v_38, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_7, ptr align 8 %valueRef_39, i64 8, i1 false)
   br label %block1
block29:
   br label %block30
block30:
   %tmp_lit_42_33 = getelementptr inbounds %struct.Int_Int, ptr %lit_42, i32 0, i32 0
   store i64 1, ptr %tmp_lit_42_33, align 8
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_7, ptr align 8 %lit_42, i64 8, i1 false)
   br label %block1
block31:
   br label %block32
block32:
   %tmp_switch_var_block32_34 = getelementptr inbounds %struct.Main_Foo, ptr %tupleField_11, i32 0, i32 0
   %tmp_switch_var_block32_35 = load i32, ptr %tmp_switch_var_block32_34, align 4
   switch i32 %tmp_switch_var_block32_35, label %block33 [

]

block33:
   %tmp_transform_46_36 = bitcast %struct.Main_Foo* %tupleField_11 to %struct.Main_Foo_Bar*
   %transform_46 = getelementptr inbounds %struct.Main_Foo_Bar, ptr %tmp_transform_46_36, i32 0, i32 1
   %tupleField_47 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %transform_46, i32 0, i32 0
   br label %block34
block34:
   %tmp_switch_var_block34_37 = getelementptr inbounds %struct.Bool_Bool, ptr %tupleField_47, i32 0, i32 0
   %tmp_switch_var_block34_38 = load i32, ptr %tmp_switch_var_block34_37, align 4
   switch i32 %tmp_switch_var_block34_38, label %block35 [
i32 1, label %block37
]

block35:
   br label %block36
block36:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v_48, ptr align 8 %tupleField_9, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_49, ptr align 8 %v_48, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_7, ptr align 8 %valueRef_49, i64 8, i1 false)
   br label %block1
block37:
   br label %block38
block38:
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %v_52, ptr align 8 %tupleField_9, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %valueRef_53, ptr align 8 %v_52, i64 8, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 8 %match_var_7, ptr align 8 %valueRef_53, i64 8, i1 false)
   br label %block1
}

define private void @Bool_Bool_True(ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Bool_Bool_True, align 4
   %tag = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 0
   store i32 1, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Bool_Bool_True, ptr %this, i32 0, i32 1
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 4, i1 false)
   ret void
}

define private void @Main_Foo_Bar(ptr noundef %f0, ptr noundef %fn_result) {
block0:
   %this = alloca %struct.Main_Foo_Bar, align 4
   %tag = getelementptr inbounds %struct.Main_Foo_Bar, ptr %this, i32 0, i32 0
   store i32 0, ptr %tag, align 4
   %payload1 = getelementptr inbounds %struct.Main_Foo_Bar, ptr %this, i32 0, i32 1
   %field0 = getelementptr inbounds %struct.siko_Tuple_Bool_Bool, ptr %payload1, i32 0, i32 0
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %field0, ptr align 4 %f0, i64 4, i1 false)
   call void @llvm.memcpy.p0.p0.i64(ptr align 4 %fn_result, ptr align 4 %this, i64 8, i1 false)
   ret void
}

declare void @String_String_eq(ptr noundef %self, ptr noundef %other, ptr noundef %fn_result)

define i32 @main() {
   %res = alloca %struct.siko_Tuple_, align 4
   call void @Main_main(ptr %res)
   ret i32 0
}


